import { Link } from "react-router-dom";

import classes from './styles.module.css';
import { useContextDiscovery } from "../../hooks/useContextDiscovery";
import { useMemo } from "react";

type GroupedContextSources = { [key: string]: string[] };

const ClusterOverview: React.FC = () => {
    const contextSources = useContextDiscovery();

    const groupedContextSources = useMemo(() => {
        const groupedContextSources: GroupedContextSources = {};

        contextSources.forEach(([source, contextName]) => {
            if (!(source in groupedContextSources)) {
                groupedContextSources[source] = [];
            }

            groupedContextSources[source].push(contextName);
        });

        return groupedContextSources;
    }, [contextSources]);

    return (
        <div className={classes.container}>
            <div>
                <h2>Your clusters</h2>
                {
                    Object.entries(groupedContextSources).map(([source, contextNames]) => (
                        <div key={source}>
                            <h6>{source}</h6>
                            <ul>
                                {
                                    contextNames.map(contextName => (
                                        <li key={contextName}>
                                            <Link to={`cluster?source=${source}&context=${contextName}`}>{contextName}</Link>
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
