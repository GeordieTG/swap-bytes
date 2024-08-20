pub mod network {
    pub mod network;
    pub mod behaviour {
        pub mod mdns;
        pub mod gossipsub;
        pub mod request_response;
        pub mod kademlia;
        pub mod rendezvous;
    }
}
pub mod state;
pub mod ui {
    pub mod page {
        pub mod global;
        pub mod rooms_menu;
        pub mod direct;
        pub mod room;
        pub mod landing;
        pub mod rating;
    }
    pub mod router;
}