import { Terminal } from "@xterm/xterm";
import { useLayoutEffect, useRef } from "react";
import AttachHyprkubeAddon from "../../xterm-addons/attach-hyprkube";
import { WebglAddon } from "@xterm/addon-webgl";
import { FitAddon } from "@xterm/addon-fit";
import { ClipboardAddon } from "@xterm/addon-clipboard";

import styles from './styles.module.css';

export interface HyprkubeTerminalProps {
    podNamespace: string,
    podName: string,
    container: string,
    clientId: string,
}

const HyprkubeTerminal: React.FC<HyprkubeTerminalProps> = (props) => {
    const xtermRef = useRef<HTMLDivElement>(null);
    const fitAddon = useRef<FitAddon | null>(null);

    useLayoutEffect(() => {
        const terminal = new Terminal({
            theme: {
                background: '#00000000',
            },
            allowTransparency: true,
        });

        fitAddon.current = new FitAddon();

        terminal.loadAddon(new AttachHyprkubeAddon(props.clientId!, props.podNamespace, props.podName, props.container));
        terminal.loadAddon(new WebglAddon());
        terminal.loadAddon(fitAddon.current);
        terminal.loadAddon(new ClipboardAddon());
        terminal.open(xtermRef.current!);

        fitAddon.current.fit();

        return () => terminal.dispose();
    });

    return (
        <div ref={xtermRef} className={styles.container}></div>
    );
};

export default HyprkubeTerminal;
