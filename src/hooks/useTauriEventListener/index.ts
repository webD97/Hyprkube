import { EventCallback, EventName, listen, Options } from "@tauri-apps/api/event";
import { useEffect, useEffectEvent } from "react";

export const useTauriEventListener = <T>(event: EventName, target: string, handler: EventCallback<T & { tabId: string }>, options?: Options) => {
    const internalHandler = useEffectEvent<EventCallback<T & { tabId: string }>>((event) => {
        if (event.payload.tabId !== target) return;
        console.log({ target, event: event.payload })
        handler(event);
    });

    useEffect(() => {
        // Cleanup may run before listen() resolves
        const unlistenPromise = listen<T & { tabId: string }>(event, (e) => internalHandler(e), options)
            .catch(e => {
                console.error(e);
                return undefined;
            });

        return () => {
            void unlistenPromise.then(unlisten => unlisten?.());
        };
    }, [event, options, target]);
};
