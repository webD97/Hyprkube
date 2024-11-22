import { Channel, invoke } from '@tauri-apps/api/core';
import { IDisposable, ITerminalAddon, Terminal } from '@xterm/xterm';

type TerminalMessage = string | { Bytes: number[] };

/**
 * Attach an xterm Terminal to a Hyprkube ExecSession
 */
export default class AttachHyprkubeAddon implements ITerminalAddon {
    private encoder = new TextEncoder();
    private disposables: IDisposable[] = [];
    private execSessionId: Promise<string> | null = null;

    constructor(private clientId: string, private podNamespace: string, private podName: string, private container: string) {
    }

    activate(terminal: Terminal): void {
        this.disposables.push(
            terminal.onData(async (data) => {
                await invoke('pod_exec_write_stdin', {
                    execSessionId: await this.execSessionId, buf: this.encoder.encode(data)
                });
            })
        );

        this.disposables.push(
            terminal.onResize(async ({ cols, rows }) => {
                console.log(`Requesting resize to ${cols}x${rows}`)
                await invoke('pod_exec_resize_terminal', {
                    execSessionId: await this.execSessionId, columns: cols, rows
                });
            })
        );

        const sessionEventChannel = new Channel<TerminalMessage>();

        sessionEventChannel.onmessage = (message) => {
            if (typeof (message) === 'object' && "Bytes" in message) {
                terminal?.write(new Uint8Array(message.Bytes));
            }
            else if (message === "End") {
                terminal?.write('\r\nHyprkube: Session exited');
            }
        };

        this.execSessionId = invoke('pod_exec_start_session', {
            clientId: this.clientId, podNamespace: this.podNamespace, podName: this.podName, container: this.container, sessionEventChannel
        });
    }

    dispose(): void {
        this.disposables.forEach(d => d.dispose());

        this.execSessionId
            ?.then(sessionId =>
                invoke('pod_exec_abort_session', {
                    execSessionId: sessionId
                })
            )
            .catch(e => console.error("Error stopping exec session", e));
    }
}
