import { listen } from "@tauri-apps/api/event";
import { useState, useEffect } from "react";
import { useLocation } from "react-router-dom";

type JoinHandleStoreStatsPayload = {
    handles: number
}

const StatusPanel: React.FC = () => {
    const [hdlCount, setHdlCount] = useState(0);
    const { pathname, search } = useLocation();

    useEffect(() => {
        listen<JoinHandleStoreStatsPayload>('join_handle_store_stats', (event) => {
            setHdlCount(event.payload.handles);
        })
    }, []);

    return (
        <>
            <div>Handles: {hdlCount}</div>
            <div>Location: {pathname}{search}</div>
        </>
    );
};

export default StatusPanel;
