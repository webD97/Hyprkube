import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

type JoinHandleStoreStatsPayload = {
    handles: number
}

const StatusPanel: React.FC = () => {
    const { handleCount } = useStats();

    return (
        <>
            <div>Channels: {handleCount}</div>
        </>
    );
};

const useStats = () => {
    const [hdlCount, setHdlCount] = useState(0);
    const cleanup = useRef<UnlistenFn>(null);

    useEffect(() => {
        listen<JoinHandleStoreStatsPayload>('join_handle_store_stats', (event) => setHdlCount(event.payload.handles))
            .then((unlisten) => cleanup.current = unlisten)
            .catch(e => console.log(e));

        return () => cleanup.current?.();
    }, []);

    return ({ handleCount: hdlCount });
}

export default StatusPanel;
