import { CSSProperties, forwardRef, ReactNode } from 'react';
import StatusBox from '../StatusBox';
import styles from './styles.module.css';

export type Component = { label: string, status: ComponentStatus };
export type ComponentStatus = 'ok' | 'warning' | 'error' | 'unknown';

export interface ClusterCardProps {
    clusterName: string,
    clusterVersion: string,
    statusStrings?: ReactNode[],
    componentsStatus?: Component[],
    onClick?: () => void,
    onAuxClick?: () => void,
    onSettingsClicked?: () => void,
    style?: CSSProperties,
    inert?: boolean,
    error?: boolean
}

const ClusterCard = forwardRef<HTMLDivElement, ClusterCardProps>(function ClusterCard(props, ref) {
    const {
        clusterName,
        clusterVersion,
        statusStrings = [],
        componentsStatus = [],
        onClick,
        onAuxClick,
        onSettingsClicked,
        style,
        inert,
        error = false
    } = props;

    return (
        <div className={`${styles.clusterCardContainer} ${error ? styles.error : ''}`} style={style} ref={ref}>
            <section className={styles.clusterCardHeader}>
                <div onClick={onClick} onAuxClick={onAuxClick} className={`${styles.cursorPointer} ${styles.titleArea}`} inert={inert}>
                    <h6>{clusterName}</h6>
                    <code className={styles.clusterVersion}>{clusterVersion}</code>
                </div>
                {
                    onSettingsClicked && (
                        <div className={styles.buttonArea}>
                            <button className={styles.settingsButton} onClick={onSettingsClicked}>⚙️</button>
                        </div>
                    )
                }
            </section>
            {
                statusStrings.length > 0 && (
                    <section>
                        {
                            statusStrings.map((status, idx) => <div key={idx}>{status}</div>)
                        }
                    </section>
                )
            }
            {
                componentsStatus.length > 0 && (
                    <section>
                        <div className={styles.componentStatusContainer}>
                            {
                                componentsStatus.map(({ label, status }, idx) => (
                                    <div key={idx} className={styles.componentStatus}>
                                        <StatusBox label={label} status={status} />
                                    </div>
                                ))
                            }
                        </div>
                    </section>
                )
            }
        </div>
    );
});

export default ClusterCard;
