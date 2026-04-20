import { confirm } from '@tauri-apps/plugin-dialog';
import { useLayoutEffect, useRef } from "react";
import ClusterSettings from '../../components/ClusterSettings';
import { KubeContextSource } from "../../hooks/useContextDiscovery";

import classes from './SettingsDialog.module.css';

export interface SettingsDialogProps {
    open?: boolean,
    contextSource: KubeContextSource,
    onAfterClose?: () => void
}

export function SettingsDialog({ open = false, contextSource, onAfterClose }: SettingsDialogProps) {
    const dialogRef = useRef<HTMLDialogElement>(null);
    const dirtyRef = useRef(false);

    useLayoutEffect(() => {
        if (open) {
            dialogRef.current?.showModal();
        }
        else {
            dialogRef.current?.close();
        }
    }, [open]);

    return (
        <dialog ref={dialogRef} className={classes.modalDialog} onClose={onAfterClose} onCancel={(e) => {
            if (dirtyRef.current) {
                e.preventDefault();

                void (async () => {
                    if (await confirm('Really?')) {
                        dialogRef.current?.close();
                    }
                })();
            }
        }}>
            <ClusterSettings contextSource={contextSource} onDirty={() => dirtyRef.current = true} onDirtyCleared={() => dirtyRef.current = false} />
        </dialog>
    );
}