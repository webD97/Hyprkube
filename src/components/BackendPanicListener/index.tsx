import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { message as messageBox } from '@tauri-apps/plugin-dialog';
import { useEffect, useRef } from "react";

interface BackendPanic {
    thread: string | null,
    location: string | null,
    message: string | null
}

const BackendPanicListener: React.FC = () => {
    const handle = useRef<UnlistenFn>(undefined);

    useEffect(() => {
        listen<BackendPanic>('background_task_panic', (event) => {
            const { thread, location, message } = event.payload;

            messageBox(`Location: ${location}\n\n${message}`, { title: `Thread '${thread}' panicked`, kind: 'error' })
                .catch(() => {
                    alert(`Thread '${thread}' panicked at ${location}.\n\n${message}`);
                })
        })
            .then(unlistenFn => handle.current = unlistenFn)
            .catch(e => alert(`Failed to setup BackendPanicListener: ${e}`));

        return () => {
            handle.current?.();
        }
    }, []);

    return null;
};

export default BackendPanicListener;
