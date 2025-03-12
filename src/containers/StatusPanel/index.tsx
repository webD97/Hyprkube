import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

type JoinHandleStoreStatsPayload = {
    handles: number
}

const StatusPanel: React.FC = () => {
    const [hdlCount, setHdlCount] = useState(0);

    useEffect(() => {
        void listen<JoinHandleStoreStatsPayload>('join_handle_store_stats', (event) => {
            setHdlCount(event.payload.handles);
        })
    }, []);

    return (
        <>
            <div>Channels: {hdlCount}</div>
        </>
    );
};

export default StatusPanel;
