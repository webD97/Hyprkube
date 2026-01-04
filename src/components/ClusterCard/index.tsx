import { ReactNode } from 'react';
import styles from './styles.module.css';

export interface ClusterCardProps {
    clusterName: string,
    clusterVersion: string,
    actions: { label: string, onTrigger: () => void, onAuxTrigger?: () => void }[],
    statusStrings?: ReactNode[],
    componentsStatus?: { label: string, color: string }[],
    availableProfiles?: string[],
    selectedProfile?: string
    onProfileChanged?: (profile: string) => void
}

const ClusterCard: React.FC<ClusterCardProps> = (props) => {
    const { clusterName, clusterVersion, actions: quickActions, statusStrings = [], componentsStatus = [], availableProfiles = [], selectedProfile, onProfileChanged = () => undefined } = props;

    return <>
        <div className={styles.clusterCardContainer}>
            <section className={styles.clusterCardHeader}>
                <div>
                    <h6>{clusterName}</h6>
                    <code>{clusterVersion}</code>
                </div>
                {
                    availableProfiles.length > 0 && (
                        <select onChange={(e) => onProfileChanged(e.target.value)} value={selectedProfile}>
                            {
                                availableProfiles.map(profile => (
                                    <option key={profile} value={profile}>{profile}</option>
                                ))
                            }
                        </select>
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
                        <div className={styles.apps}>
                            {
                                componentsStatus.map(({ label, color }, idx) => (
                                    <div key={idx} className={styles.appSummary}>
                                        <span className={styles.appName}>{label}</span>
                                        <span className={styles.appStatusBox} style={{ backgroundColor: color }}>&nbsp;</span>
                                    </div>
                                ))
                            }
                        </div>
                    </section>
                )
            }
            <section className={styles.clusterCardActions}>
                {
                    quickActions.map(({ label, onTrigger, onAuxTrigger }) => (
                        <button key={label} onClick={onTrigger} onAuxClick={onAuxTrigger}>{label}</button>
                    ))
                }
            </section>
        </div>
    </>;
}

export default ClusterCard;
