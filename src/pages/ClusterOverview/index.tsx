import { Link } from "react-router-dom";

import { useMemo } from "react";
import { KubeContextSource, useContextDiscovery } from "../../hooks/useContextDiscovery";
import classes from './styles.module.css';

type GroupedContextSources = {
    [key: string]: {
        contexts: KubeContextSource[]
    }
};

const ClusterOverview: React.FC = () => {
    const contextSources = useContextDiscovery();

    const groupedContextSources = useMemo(() => {
        const groupedContextSources: GroupedContextSources = {};

        contextSources.forEach((contextSource) => {
            const { provider, source } = contextSource;
            let displayName = provider + "://" + source;

            if (source.includes('Lens/')) {
                displayName = source.substring(0, source.lastIndexOf('/'));
            }

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
                                    contextGroup.contexts.map(({ source, context }, idx) => (
                                        <li key={idx}>
                                            <Link to={`cluster?source=${source}&context=${context}`}>{context}</Link>
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
