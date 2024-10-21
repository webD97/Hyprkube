import { listen } from "@tauri-apps/api/event";
import { useState, useEffect } from "react";

type JoinHandleStoreStatsPayload = {
    channels: number,
    handles: number
}

// eslint-disable-next-line @typescript-eslint/no-empty-object-type
const StatusPanel: React.FC<{}> = () => {
    const [chanCount, setChanCount] = useState(0);
    const [hdlCount, setHdlCount] = useState(0);

    useEffect(() => {
        listen<JoinHandleStoreStatsPayload>('join_handle_store_stats', (event) => {
            setChanCount(event.payload.channels);
            setHdlCount(event.payload.handles);
        })
    }, []);

    return (
        <>
            <div>Channels: {chanCount}</div>
            <div>Handles: {hdlCount}</div>
        </>
    );
};

export default StatusPanel;
