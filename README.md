# Swapbytes 📚
A peer to peer file sharing application using libp2p for COSC473 @ UC. 
Designed for students to join chat rooms and organise the trading of study notes for various classes.

## How to run

- Start a Peer using ```cargo run --bin peer```

## How to Use

### Enter a Nickname 🥷
On start you will be prompted to enter a nickname. This will be your display name to all users throughout the application. NOTE: This can't be changed!

<img width="1132" alt="image" src="https://github.com/user-attachments/assets/6f0c667b-36f4-4fb9-95c0-cc6ff16f81a1">

### Global Chat 🌍
The global chat is where all connected users can communicate with each other. Simply type your message and press enter to send. 

<img width="1000" alt="image" src="https://github.com/user-attachments/assets/9ce69e1c-2897-4c0f-9881-f73409ba7e3c">

### Rooms 🏘️
Rooms are dedicated spaces for peers to communicate about specific topics. On this tab users will have a selection of existing rooms to join, or they have the option of creating their own room to be shared across the network.

<img width="500" alt="image" src="https://github.com/user-attachments/assets/d45a98fd-a0d2-4cb6-a78d-c26e176966e3">
<img width="500" alt="image" src="https://github.com/user-attachments/assets/1c761e31-bd74-48d0-8a67-e4b3c4e66c64">

### Requesting a File ✨
After agreeing on a trade within a chat room, the user is able to request a file from a peer on the "Direct Messages" tab. Simply find the user in the "Request File" list and press enter. You will be prompted to add a request message. Something such as "Please give me the COSC473 notes for last week" for example. On enter this will send a request to the user.

### Sending a File 🚀
When receiving a file request, it will appear under the "Incoming Request" list in the "Direct Messages" tab. It will appear as ```<user> - <request message>```. To address the request, navigate to it using the arrow keys and press enter. You will be prompted to enter a filepath of the file you want to send.

### Receiving a File 💁‍♀️
In the event another user sends you a file, you will receive a notification through a pop up. The file will be downloaded to your computer as "swapbytes.txt" and you will be asked to rate the user depending on whether they sent you what you asked for (Good, Neutral or Bad).

<img width="1000" alt="image" src="https://github.com/user-attachments/assets/0b6b94b3-e3cb-4fbd-8b34-1b6d3380e5c5">


### Peer Ratings 📊
Peer ratings are a way of acknowledging users who act morally or immorally on the platform. All users start with a peer rating of 0, and will recieve +1 for each "Good" rating and -1 for each "Bad" rating. When a user types in the chat, their messages will display an emoji to indicate to other users how reliable they are. 

- 👿 = Peer with a rating lower than 0
- 😇 = Peer with a rating higher than 0

The users exact rating is visisble next to their name in the "Request a File" list on the "Direct Messages" tab.

<img width="994" alt="image" src="https://github.com/user-attachments/assets/cb6679e5-a505-4f7c-a61e-71366ab35ced">

### Notifications 🔔
When the user receives a message / file request from any user / room, they need to be notified! 

- When the user receives a message:
  The "Rooms" tab will display with a "🔔" and the specific room / direct message will display with "- New Messages" next to it.

- When the user receives a file request:
  The "File Sharing" tab will display with a "🔔"

