pub mod network {
    pub mod network;
    pub mod client;
    pub mod behaviour {
        pub mod mdns;
        pub mod gossipsub;
        pub mod request_response;
        pub mod kademlia;
    }
}
pub mod ui {
    pub mod page {
        pub mod chat;
        pub mod rooms_menu;
        pub mod direct;
        pub mod landing;
        pub mod rating;
    }
    pub mod router;
    pub mod components;
}

pub mod state;