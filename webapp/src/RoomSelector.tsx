import * as React from 'react';
export type Fn<T, R=void> = (x:T) => R;

export const RoomSelector = ({ rooms, onSelect }: { rooms: string[], onSelect: Fn<string> }) => {
    return (
        <React.Fragment>
            {rooms.map(room => (
                <button key={room} onClick={() => onSelect(room)}>{room}</button>
            ))}
        </React.Fragment>
    )
};
