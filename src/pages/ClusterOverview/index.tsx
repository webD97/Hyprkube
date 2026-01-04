import { use, useMemo } from "react";
import ClusterCard from "../../components/ClusterCard";
import MegaTabsContext from "../../contexts/MegaTabs";
import { KubeContextSource, useContextDiscovery } from "../../hooks/useContextDiscovery";
import { capitalizeFirstLetter } from "../../utils/strings";
import ClusterView from "../ClusterView";
import classes from './styles.module.css';

type GroupedContextSources = {
    [key: string]: {
        contexts: KubeContextSource[]
    }
};

const ClusterOverview: React.FC = () => {
    const contextSources = useContextDiscovery();
    const { pushTab, switchTab, replaceActiveTab } = use(MegaTabsContext)!;

    const groupedContextSources = useMemo(() => {
        const groupedContextSources: GroupedContextSources = {};

        contextSources.forEach((contextSource) => {
            const { provider, source } = contextSource;
            let displayName = source;

            if (source.includes('Lens/')) {
                displayName = source.substring(0, source.lastIndexOf('/'));
            }

            displayName = provider + "://" + displayName;

            if (!(displayName in groupedContextSources)) {
                groupedContextSources[displayName] = {
                    contexts: []
                };
            }

            groupedContextSources[displayName].contexts.push(contextSource);
        });

        return groupedContextSources;
    }, [contextSources]);

    return (
        <div className={classes.container}>
            <h2>Your clusters</h2>
            <div>
                {
                    Object.entries(groupedContextSources).map(([source, contextGroup]) => (
                        <div key={source}>
                            <h4>{source}</h4>
                            {
                                contextGroup.contexts.map((contextSource, idx) => (
                                    <ClusterCard key={idx}
                                        clusterName="Homelab"
                                        clusterVersion="v1.34.2+k3s1"
                                        availableProfiles={["Individual profile", "Temporary profile"]}
                                        selectedProfile={'Temporary profile'}
                                        onProfileChanged={(profile) => alert(`Profile changed to: ${profile}`)}
                                        statusStrings={[
                                            <>144 Pods, 16 <code style={{ color: 'crimson' }}>Failed</code>, 24 <code style={{ color: 'lightgreen' }}>Succeeded</code></>,
                                            <>5 nodes, 1 <code style={{ color: 'orange' }}>NotReady</code></>,
                                            <>42 Bundles, 2 <code style={{ color: 'orange' }}>NotReady</code></>
                                        ]}
                                        componentsStatus={[
                                            { label: 'Jellyfin', color: 'lime' },
                                            { label: 'Homeassistant', color: 'lime' },
                                            { label: 'PiHole', color: 'lime' },
                                            { label: 'PaperlessNGX', color: 'lime' },
                                            { label: 'Octoprint', color: 'orange' },
                                        ]}
                                        actions={[
                                            {
                                                label: '📖 Connect',
                                                onAuxTrigger() {
                                                    switchTab(pushTab(
                                                        { title: capitalizeFirstLetter(contextSource.context), icon: '🌍', keepAlive: true },
                                                        () => <ClusterView contextSource={contextSource} />
                                                    ));
                                                },
                                                onTrigger() {
                                                    replaceActiveTab(
                                                        { title: capitalizeFirstLetter(contextSource.context), icon: '🌍', keepAlive: true },
                                                        () => <ClusterView contextSource={contextSource} />
                                                    );
                                                }
                                            },
                                            { label: '📝 Edit Kubeconfig', onTrigger: () => alert('Edit Kubeconfig') },
                                            { label: '🔃 Sync Fleet', onTrigger: () => alert('Sync Fleet') },
                                        ]}
                                    />
                                ))
                            }
                        </div>
                    ))
                }
            </div>
        </div>
    );
};

export default ClusterOverview;
