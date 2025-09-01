use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use rust_socketio::Payload;
use rust_socketio::{ClientBuilder, client::Client};
use mpv_client_dyn::{mpv_handle, osd, Event, Format, Handle, Property};
use serde::{Serialize, Deserialize};
use serde_json::json;
use crate::config::Config;

// observe_property requires a "reply" arg which is used to match responses to requests
const PAUSE_PROPERTY_ID: u64 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaPlayerEvent {
    Play,
    Pause,
    Seeked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomEvent {
    pub location: String,
    #[serde(rename = "type")]
    pub event_type: MediaPlayerEvent,
    pub element: u32,
    pub current_time: f64,
    pub playback_rate: f64,
}

impl RoomEvent {
    pub fn new(
        location: String,
        event_type: MediaPlayerEvent,
        element: u32,
        current_time: f64,
        playback_rate: f64,
    ) -> Self {
        Self {
            location,
            event_type,
            element,
            current_time,
            playback_rate,
        }
    }
}

pub struct Plugin {
    handle: *mut mpv_handle,
    config: Config,
    is_enabled: bool,
    socket: Option<Client>,
    pause_counter: Arc<Mutex<u32>>,
}

impl Plugin {
    pub fn new(handle: *mut mpv_handle, config: Config) -> Self {
        Plugin {
            handle,
            config: config,
            is_enabled: false,
            socket: None,
            pause_counter: Arc::new(Mutex::new(1)),
        }
    }

    pub fn start(&mut self) -> Result<()> {
        if self.config.enable_on_start {
            self.enable()?;
        }

        let mut handle = Handle::from_ptr(self.handle);
        let pause_counter = self.pause_counter.clone();

        let socket = ClientBuilder::new(&self.config.server_url)
            .transport_type(rust_socketio::TransportType::Websocket)
            .on("message", move |payload, _| {
                Self::handle_message_static(&mut handle, payload, &pause_counter);
            })
            .connect()?;

        // https://github.com/1c3t3a/rust-socketio/issues/502
        std::thread::sleep(Duration::from_millis(500));

        socket.emit("join", json!({
            "name": &self.config.name,
            "room": &self.config.room_name,
        }))?;

        self.socket = Some(socket);

        loop {
            let result = match self.wait_event(-1.0) {
                Event::Shutdown => break,
                Event::PropertyChange(PAUSE_PROPERTY_ID, property) => self.handle_pause_unpause(property),
                Event::ClientMessage(data) => self.handle_client_message(data),
                _ => Ok(()),
            };

            if let Err(e) = result {
                log::error!("Error handling event: {}", e);
            }
        }

        Ok(())
    }

    // This is called from a separate thread
    fn handle_message_static(handle: &mut Handle, payload: Payload, pause_counter: &Arc<Mutex<u32>>) {
        log::trace!("Received payload: {:?}", payload);

        let Payload::Text(payload) = payload else {
            log::warn!("Received non-text payload");
            return;
        };

        let Some(payload) = payload.first() else {
            log::warn!("Received empty text payload");
            return;
        };

        let room_event = match serde_json::from_value::<RoomEvent>(payload.clone()) {
            Ok(event) => event,
            Err(e) => {
                log::warn!("Failed to parse RoomEvent from payload: {}", e);
                return;
            }
        };

        handle.set_property("time-pos", room_event.current_time).ok();

        pause_counter.lock().map(|mut counter| {
            *counter += 1;
        }).ok();

        handle.set_property("pause", matches!(room_event.event_type, MediaPlayerEvent::Pause)).ok();
    }
    
    fn enable(&mut self) -> Result<()> {
        log::trace!("Enabling syncwatch");

        if !self.is_enabled {
            self.is_enabled = true;

            // Set up property observes
            self.observe_property(PAUSE_PROPERTY_ID, "pause", bool::MPV_FORMAT)?;

            osd!(self, Duration::from_secs(2), "syncwatch enabled")?;
        }

        Ok(())
    }

    fn disable(&mut self) -> Result<()> {
        log::trace!("Disabling syncwatch");

        if self.is_enabled {
            self.is_enabled = false;

            // Tear down property observes
            self.unobserve_property(PAUSE_PROPERTY_ID)?;

            osd!(self, Duration::from_secs(2), "syncwatch disabled")?;
        }

        Ok(())
    }
    
    fn toggle(&mut self) -> Result<()> {
        if self.is_enabled {
            self.disable()
        } else {
            self.enable()
        }
    }

    fn handle_pause_unpause(&mut self, property: Property) -> Result<()> {
        let Some(is_paused) = property.data::<bool>() else {
            log::warn!("Pause property change with non-bool data");
            return Ok(());
        };

        log::trace!("Pause state changed: {}", is_paused);

        let mut pause_counter = self.pause_counter.lock().unwrap();
        
        if *pause_counter > 0 {
            log::trace!("Ignoring pause/unpause event");
            *pause_counter -= 1;
            return Ok(());
        }

        drop(pause_counter);

        let current_time = self.get_property("time-pos")?;

        let Some(socket) = &self.socket else {
            log::warn!("No socket connection to server");
            return Ok(());
        };

        let event_type = if is_paused { MediaPlayerEvent::Pause } else { MediaPlayerEvent::Play };

        let event = RoomEvent::new(
            self.config.room_name.clone(),
            event_type.clone(),
            0,
            current_time,
            0.0,
        );

        log::trace!("Emitting {:?} event: {:?}", event_type, event);

        socket.emit("message", serde_json::to_value(event)?)?;

        Ok(())
    }
    
    fn handle_client_message(&mut self, data: mpv_client_dyn::ClientMessage) -> Result<()> {
        log::trace!("Received client message: {}", data.args().join(" "));

        match data.args().as_slice() {
            ["key-binding", "toggle", "u--", ..] => self.toggle(),
            _ => Ok(()),
        }
    }
}

impl Deref for Plugin {
    type Target = Handle;

    fn deref(&self) -> &Self::Target {
        Handle::from_ptr(self.handle)
    }
}

impl DerefMut for Plugin {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Handle::from_ptr(self.handle)
    }
}