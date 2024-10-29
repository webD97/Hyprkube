import { listen } from "@tauri-apps/api/event";
import { useState, useEffect } from "react";

type JoinHandleStoreStatsPayload = {
    handles: number
}

// eslint-disable-next-line @typescript-eslint/no-empty-object-type
const StatusPanel: React.FC<{}> = () => {
    const [hdlCount, setHdlCount] = useState(0);

    useEffect(() => {
        listen<JoinHandleStoreStatsPayload>('join_handle_store_stats', (event) => {
            setHdlCount(event.payload.handles);
        })
    }, []);

    return (
        <>
            <div>Handles: {hdlCount}</div>
        </>
    );
};

export default StatusPanel;
