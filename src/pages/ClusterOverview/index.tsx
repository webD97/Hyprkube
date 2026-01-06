import { useContext, useMemo } from "react";
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
    const { replaceActiveTab } = useContext(MegaTabsContext)!;

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
                            <ul className={classes.clusterList}>
                                {
                                    contextGroup.contexts.map((contextSource, idx) => (
                                        <li key={idx}>
                                            <a href="" onClick={(e) => {
                                                e.preventDefault();

                                                replaceActiveTab(
                                                    { title: capitalizeFirstLetter(contextSource.context), icon: '🌍', keepAlive: true },
                                                    () => <ClusterView contextSource={contextSource} />
                                                );
                                            }}>{contextSource.context}</a>
                                        </li>
                                    ))
                                }
                            </ul>
                        </div>
                    ))
                }
            </div>
        </div>
    );
};

export default ClusterOverview;
