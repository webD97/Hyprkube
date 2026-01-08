import styles from './styles.module.css';

export type ComponentStatus = 'ok' | 'warning' | 'error' | 'unknown';

export interface StatusBoxProps {
    status: ComponentStatus,
    label?: string,
}

export default function StatusBox({ label, status }: StatusBoxProps) {
    return (
        <span title={label} className={styles.statusBox} data-status={status}>&nbsp;</span>
    );
}
