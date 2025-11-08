import { EventCallback, EventName, listen, Options, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

export const useTauriEventListener = <T>(event: EventName, handler: EventCallback<T>, options?: Options) => {
    const unlisten = useRef<UnlistenFn>(null);

    useEffect(() => {
        console.log("registered");

        listen<T>(event, handler, options)
            .then(u => unlisten.current = u)
            .catch(e => console.error(e));

        return () => unlisten.current?.();
    }, [event, handler, options]);
};
