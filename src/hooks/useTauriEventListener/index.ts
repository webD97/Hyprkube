import { EventCallback, EventName, listen, Options, UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef } from "react";

export const useTauriEventListener = <T>(event: EventName, target: string, handler: EventCallback<T & { tabId: string }>, options?: Options) => {
    const unlisten = useRef<UnlistenFn>(null);

    const internalHandler = useCallback<EventCallback<T & { tabId: string }>>((event) => {
        if (event.payload.tabId !== target) return;
        console.log({ target, event: event.payload })
        handler(event);
    }, [handler, target]);

    useEffect(() => {
        listen<T & { tabId: string }>(event, internalHandler, options)
            .then(u => unlisten.current = u)
            .catch(e => console.error(e));

        return () => unlisten.current?.();
    }, [event, handler, internalHandler, options, target]);
};
