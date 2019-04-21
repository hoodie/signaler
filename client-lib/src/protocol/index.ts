import * as command from './command';
import { Command } from './command';
import * as serverEvent from './event';
import { ServerEvent } from './event';

export { command, serverEvent, Command, ServerEvent };

export const isWelcomeEvent = (msg: any): msg is serverEvent.Welcome =>
    typeof msg === 'object' && msg.type === 'welcome' && isSessionDescription(msg.session);

export interface SessionDescription {
    session_id: string;
}

export const isSessionDescription = (d: any) => typeof d === 'object' && typeof d.session_id === 'string';

export interface ChatMessage {
    content: string;
    sender: string;
    uuid: string;
    received: Date;
}

export interface UserProfile {
    fullName: string;
}
