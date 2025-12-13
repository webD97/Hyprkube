import { IDisposable, ITerminalAddon, Terminal } from '@xterm/xterm';
import { abortExecSession, resizeTerminal, startExecSession, writeBytes } from '../../api/podExec';
import { KubeContextSource } from '../../hooks/useContextDiscovery';

/**
 * Attach an xterm Terminal to a Hyprkube ExecSession
 */
export default class AttachHyprkubeAddon implements ITerminalAddon {
    private encoder = new TextEncoder();
    private disposables: IDisposable[] = [];
    private execSessionId: Promise<string> | null = null;

    constructor(private contextSource: KubeContextSource, private podNamespace: string, private podName: string, private container: string) {
    }

    activate(terminal: Terminal): void {
        this.disposables.push(
            terminal.onData(async (data) => {
                await this.execSessionId
                    ?.then(sessionId => writeBytes(sessionId, this.encoder.encode(data)))
                    .catch(e => console.log("Failed to write to terminal:", e));
            })
        );

        this.disposables.push(
            terminal.onResize(async ({ cols, rows }) => {
                console.log(`Requesting resize to ${cols}x${rows}`);
                await this.execSessionId
                    ?.then(sessionId => resizeTerminal(sessionId, cols, rows))
                    .catch(e => console.log("Failed to resize terminal:", e));
            })
        );

        const [sessionIdPromise, sessionEventChannel] = startExecSession(this.contextSource, this.podNamespace, this.podName, this.container);

        this.execSessionId = sessionIdPromise;

        sessionEventChannel.onmessage = (message) => {
            if (typeof (message) === 'object' && "Bytes" in message) {
                terminal?.write(new Uint8Array(message.Bytes));
            }
            else if (message === "End") {
                terminal?.write('\r\nHyprkube: Session exited');
            }
        };
    }

    dispose(): void {
        this.disposables.forEach(d => d.dispose());

        this.execSessionId
            ?.then(abortExecSession)
            .catch(e => console.error("Error stopping exec session", e));
    }
}
