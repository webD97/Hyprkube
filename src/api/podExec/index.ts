import { Channel, invoke } from "@tauri-apps/api/core";

export type UpstreamTerminalMessage = string | { Bytes: number[] };

export function startExecSession(clientId: string, podNamespace: string, podName: string, container: string): [Promise<string>, Channel<UpstreamTerminalMessage>] {
    const sessionEventChannel = new Channel<UpstreamTerminalMessage>();

    const sessionIdPromise: Promise<string> = invoke('pod_exec_start_session', {
        clientId, podNamespace, podName, container, sessionEventChannel
    });

    return [sessionIdPromise, sessionEventChannel];
}

export function abortExecSession(sessionId: string) {
    return invoke('pod_exec_abort_session', {
        execSessionId: sessionId
    });
}

export function resizeTerminal(sessionId: string, columns: number, rows: number): Promise<void> {
    return invoke('pod_exec_resize_terminal', {
        execSessionId: sessionId, columns, rows
    });
}

export function writeBytes(sessionId: string, bytes: Uint8Array) {
    return invoke('pod_exec_write_stdin', {
        execSessionId: sessionId, buf: bytes
    });
}
