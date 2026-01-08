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

export default function Welcome() {
    const contextSources = useContextDiscovery();
    const { replaceActiveTab } = use(MegaTabsContext)!;

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
        <div className={classes.welcomeContainer}>
            <h2>Your clusters</h2>
            <div>
                {
                    Object.entries(groupedContextSources).map(([source, contextGroup]) => (
                        <div key={source}>
                            <h4>{source}</h4>
                            <div className={classes.clusterList}>
                                {
                                    contextGroup.contexts.map((contextSource, idx) => (
                                        <ClusterCard key={idx}
                                            clusterName={capitalizeFirstLetter(contextSource.context)}
                                            clusterVersion="v1.34.2+k3s1"
                                            onConnect={() => replaceActiveTab(
                                                { title: capitalizeFirstLetter(contextSource.context), icon: '🌍', keepAlive: true },
                                                () => <ClusterView contextSource={contextSource} />
                                            )}
                                        />
                                    ))
                                }
                            </div>
                        </div>
                    ))
                }
            </div>
        </div>
    );
};
