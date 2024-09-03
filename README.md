# Swapbytes 📚
A peer to peer file sharing application using libp2p for COSC473 @ UC. 
Designed for students to join chat rooms and organise the trading of study notes for various classes.

## How to run

- Start a Peer using ```cargo run --bin peer```

## Main Controls 🕹️

- Tab -> Cycle Through Tabs
- Up and Down Arrows -> Navigate through lists
- Left and Right Arrows -> Jump between left and right sections (Rooms / File Sharing Tabs)
- Characters / Numbers -> Input
- Backspace -> Delete input characters
- Enter -> Used to 1) select items from list and 2) confirm inputs
- Esc -> Close application

## How to Use

### Enter a Nickname 🥷
On start you will be prompted to enter a nickname. This will be your display name to all users throughout the application. NOTE: This can't be changed!

<img width="1119" alt="image" src="https://github.com/user-attachments/assets/689a6199-ee98-4006-9f48-a80edd1f93c4">

### Chat 🌍
The chat is where all connected users can communicate with each other. Simply type your message and press enter to send. 

<img width="1116" alt="image" src="https://github.com/user-attachments/assets/ca3d58ba-df54-4dbd-9ce4-b91b2bc573a6">

### Rooms & Direct Messages 🏘️

Rooms are dedicated spaces for peers to communicate about specific topics. On this tab users will have a selection of existing rooms to join, or they have the option of creating their own room to be shared across the network.

On the right hand side of the page, users also have the chance to start a direct message with any other peer on the network.

When any of these options are selected, the user is navigated back to the "Chat" tab where they will then be communicating within the selected chat. The user can change their chat at anytime through the "Rooms" tab.

<img width="995" alt="image" src="https://github.com/user-attachments/assets/fe8f10ad-1612-4093-a14e-3405ae97042a">
<img width="992" alt="image" src="https://github.com/user-attachments/assets/67b235c5-c811-4cd5-b3cd-fca001b00d12">

### Requesting a File ✨
After agreeing on a trade within a chat room, the user is able to request a file from a peer on the "File Sharing" tab. Simply find the user in the "Request File" list and press enter. You will be prompted to add a request message. On enter this will send a request to the user.

<img width="996" alt="image" src="https://github.com/user-attachments/assets/8b38dc7b-a847-4270-8eff-51c8347a278a">

### Sending a File 🚀
When receiving a file request, it will appear under the "Incoming Request" list in the "File Sharing" tab. It will appear as ```<user> - <request message>```. To address the request, navigate to it using the arrow keys and press enter. You will be prompted to enter a filepath of the file you want to send.

<img width="993" alt="image" src="https://github.com/user-attachments/assets/43f4778f-0a57-4697-a75f-1356f428025c">

### Receiving a File 💁‍♀️
In the event another user sends you a file, you will receive a notification through a pop up. The file will be downloaded to your computer as "swapbytes.txt" and you will be asked to rate the user depending on whether they sent you what you asked for (Good, Neutral or Bad).

<img width="1000" alt="image" src="https://github.com/user-attachments/assets/0b6b94b3-e3cb-4fbd-8b34-1b6d3380e5c5">

### Peer Ratings 📊
Peer ratings are a way of acknowledging users who act morally or immorally on the platform. All users start with a peer rating of 0, and will recieve +1 for each "Good" rating and -1 for each "Bad" rating. These exact ratings are hidden but are stored on the network. When a user types in the chat, their messages will display an emoji to indicate to other users how reliable they are. 

- 👿 = Peer with a rating lower than 0
- 😇 = Peer with a rating higher than 0

<img width="995" alt="image" src="https://github.com/user-attachments/assets/270e65ba-5d5e-453a-9543-8b44bcbe8b85">

### Notifications 🔔
When the user receives a message / file request from any user / room, they need to be notified! 

- When the user receives a message:
  The "Rooms" tab will display with a "🔔" and the specific room / direct message will display with "- New Messages" next to it.

- When the user receives a file request:
  The "File Sharing" tab will display with a "🔔"

These notifications disappear when the messages are read, or the request is responded to. It is also important to note that because newly created rooms are only checked on the "Rooms" page, newly created rooms won't give notifications until you have visited the "Rooms" tab and fetched the latest rooms.

<img width="1122" alt="image" src="https://github.com/user-attachments/assets/e8bc8664-7ffa-4fd4-91d6-bda3e32096d3">
<img width="998" alt="image" src="https://github.com/user-attachments/assets/482891b9-e5bc-4fc3-b507-b2dced1b5d5e">


