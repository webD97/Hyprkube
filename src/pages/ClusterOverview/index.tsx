import { Link } from "react-router-dom";

import classes from './styles.module.css';
import { useContextDiscovery } from "../../hooks/useContextDiscovery";
import { useMemo } from "react";

type GroupedContextSources = {
    [key: string]: {
        contexts: [string,string][]
    }
};

const ClusterOverview: React.FC = () => {
    const contextSources = useContextDiscovery();

    const groupedContextSources = useMemo(() => {
        const groupedContextSources: GroupedContextSources = {};

        contextSources.forEach(([source, contextName]) => {
            let displayName = source;

            if (source.includes('Lens/')) {
                displayName = source.substring(0, source.lastIndexOf('/'));
            }

            if (!(displayName in groupedContextSources)) {
                groupedContextSources[displayName] = {
                    contexts: []
                };
            }

            groupedContextSources[displayName].contexts.push([source, contextName]);
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
                                    contextGroup.contexts.map(([source, name], idx) => (
                                        <li key={idx}>
                                            <Link to={`cluster?source=${source}&context=${name}`}>{name}</Link>
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
