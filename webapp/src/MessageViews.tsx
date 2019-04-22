import * as React from 'react';
import { ChatMessage } from "../../client-lib/src";

export const MessageList = ({ messages, me }: { messages: ChatMessage[], me?: string }) => <div className="messageList">
        { messages.map(msg => <ChatMessageView message={msg} me={me} key={msg.uuid} />) }
    </div>

export const ChatMessageView = ({ message, me }: { message: ChatMessage, me?: string }) => {
    const senderName = message.sender === me ? 'me' : message.senderName;
    return (<React.Fragment>
        <span className="timestamp">{message.sent.getHours()}:{message.sent.getMinutes()}</span>
        <span className={`sender ${senderName}`}>{senderName}</span>
        <span className="content">{message.content}</span>
    </React.Fragment>);
};
