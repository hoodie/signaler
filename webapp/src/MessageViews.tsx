import * as React from 'react';
import { ChatMessage } from "../../client-lib/src";
import { Participant, RoomParticipants } from '../../client-lib/src/protocol';

interface MessageListProps {
    messages: ChatMessage[],
    me?: string,
    participants: Array<Participant>
}

export const MessageList = ({ messages, me, participants }: MessageListProps) => <div className="messageList">
        {messages.map(msg => <ChatMessageView message={msg} me={me} key={msg.uuid} participants={participants} />)}
    </div>

export const ChatMessageView = ({ message, me, participants }: { message: ChatMessage, me?: string, participants: Array<Participant> }) => {
    const senderName = participants.find(participant => participant.sessionId === message.sender)?.fullName || 'unknown participant';
    return (<React.Fragment>
        <span className="timestamp">{message.sent.getHours()}:{message.sent.getMinutes()}</span>
        <span className={`sender ${senderName}`}>{senderName}</span>
        <span className="content">{message.content}</span>
    </React.Fragment>);
};
