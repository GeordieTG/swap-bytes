use libp2p_request_response::ResponseChannel;
use libp2p::PeerId;
use futures::channel::mpsc;
use futures::SinkExt;

use super::{command::Command, network::Response};

/// Used to send commands from the UI to the Network.
/// For example if a user types a message in the UI to send to the global chat, we must instruct the libp2p
/// network to send our message. mpsc allows us to send these commands across asyncronous tasks.
#[derive(Clone)]
pub struct Client {
    pub sender: mpsc::Sender<Command>,
}

impl Client {
    
    /// Send a given message the user has typed in the UI.
    pub(crate) async fn send_message(
        &mut self,
        message: String,
        room: String
    ) {
        self.sender
            .send(Command::SendMessage { message, room })
            .await
            .expect("Command receiver not to be dropped.");
    }


    /// Request a file from another user.
    pub(crate) async fn send_request(
        &mut self,
        message: String,
        peer: PeerId
    ) {
        self.sender
            .send(Command::RequestFile { message, peer })
            .await
            .expect("Command receiver not to be dropped.");
    }

    
    /// Send the file at the given path back to the user who requested it.
    pub(crate) async fn send_response(
        &mut self,
        filename: String,
        filepath: String,
        channel: ResponseChannel<Response>
    ) {
        self.sender
            .send(Command::RespondFile { filename, filepath, channel })
            .await
            .expect("Command receiver not to be dropped.");
    }

    /// Update the rating of another peer.
    /// Called after giving a peer a rating during a file swap to either increase or decrease their rating by 1.
    pub(crate) async fn update_rating (
        &mut self,
        peer: PeerId,
        rating: i32,
    ) {
        self.sender
            .send(Command::UpdateRating { peer, rating })
            .await
            .expect("Command receiver not to be dropped.");
    }


    /// Create a new room to be shared across the network
    pub(crate) async fn create_room (
        &mut self,
        name: String,
    ) {
        self.sender
            .send(Command::CreateRoom { name })
            .await
            .expect("Command receiver not to be dropped.");
    }


    /// Fetch all currently available rooms to join
    pub(crate) async fn fetch_rooms (
        &mut self,
    ) {
        self.sender
            .send(Command::FetchRooms {  })
            .await
            .expect("Command receiver not to be dropped.");
    }
}