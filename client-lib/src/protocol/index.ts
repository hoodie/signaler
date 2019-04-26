import * as command from './command';
import { Command } from './command';
import * as serverEvent from './event';
import { ServerEvent, RoomParticipants } from './event';

export { command, serverEvent, Command, ServerEvent, RoomParticipants };

export const isWelcomeEvent = (msg: any): msg is serverEvent.Welcome => 
    typeof msg === 'object' && msg.type === 'welcome' && isSessionDescription(msg.session);


export interface SessionDescription {
    sessionId: string;
}

export const isSessionDescription = (d: serverEvent.Welcome['session']) => typeof d === 'object' && typeof d.sessionId === 'string';

export interface ChatMessage {
    content: string;
    sender: string;
    sent: Date,
    senderName: string;
    uuid: string;
    received: Date;
}

export interface RawChatMessage  {
    content: string;
    sender: string;
    sent: string,
    senderName: string;
    uuid: string;
}

export interface UserProfile {
    fullName: string;
}

export interface Participant {
    fullName: string;
    sessionId: string;
}
