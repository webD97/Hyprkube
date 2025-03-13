import { getCurrentWindow } from '@tauri-apps/api/window';
import { useCallback, useMemo } from 'react';
import classes from './styles.module.css';


export const WindowControls: React.FC = () => {
    const currentWindow = useMemo(getCurrentWindow, []);
    const minimize = useCallback(() => void currentWindow.minimize(), [currentWindow]);
    const maximize = useCallback(() => void currentWindow.toggleMaximize(), [currentWindow]);
    const close = useCallback(() => void currentWindow.close(), [currentWindow]);

    return (
        <div className={classes.container}>
            <div className={`${classes.button} ${classes.minimize}`} onClick={minimize}>
                <svg className={classes.icon} viewBox="0 0 12 12"><rect y="6" width="12" height="2" /></svg>
            </div>
            <div className={`${classes.button} ${classes.maximize}`} onClick={maximize}>
                <svg className={classes.icon} viewBox="0 0 12 12"><rect x="2" y="2" width="8" height="8" /></svg>
            </div>
            <div className={`${classes.button} ${classes.close}`} onClick={close}>
                <svg className={classes.icon} viewBox="0 0 12 12">
                    <line x1="2" y1="2" x2="10" y2="10" stroke="white" strokeWidth="2" />
                    <line x1="10" y1="2" x2="2" y2="10" stroke="white" strokeWidth="2" />
                </svg>
            </div>
        </div>
    );
};