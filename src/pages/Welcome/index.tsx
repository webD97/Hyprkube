import { useQuery } from "@tanstack/react-query";
import { use, useRef, useState } from "react";
import ClusterCard from "../../components/ClusterCard";
import MegaTabsContext from "../../contexts/MegaTabs";
import { KubeContextSource } from "../../hooks/useContextDiscovery";
import useIntersectionObserver from "../../hooks/useIntersectionObserver";
import discoverContextsQuery from "../../queries/discoverContexts";
import getApiServerGitVersionQuery from "../../queries/getApiServerGitVersion";
import { capitalizeFirstLetter } from "../../utils/strings";
import ClusterView from "../ClusterView";
import { SettingsDialog } from "./SettingsDialog";
import classes from './styles.module.css';

export default function Welcome() {
    const { replaceActiveTab, pushTab } = use(MegaTabsContext)!;
    const [currentSettingsCluster, setCurrentSettingsCluster] = useState<KubeContextSource | null>(null);

    const contextSources = useQuery({
        ...discoverContextsQuery(),
        placeholderData: []
    });

    return (
        <>
            <div className={classes.welcomeContainer}>
                <h2>Your clusters</h2>
                <div>
                    {
                        Object.entries(contextSources.data!).map(([source, contextGroup]) => (
                            <div key={source}>
                                <h4>{source}</h4>
                                <div className={classes.clusterList}>
                                    {
                                        contextGroup.contexts.map((contextSource, idx) => (
                                            <ClusterCardWithInfo key={idx}
                                                contextSource={contextSource}
                                                onConnect={() => replaceActiveTab(
                                                    { title: capitalizeFirstLetter(contextSource.context), icon: '🌍', keepAlive: true },
                                                    () => <ClusterView contextSource={contextSource} />
                                                )}
                                                onConnectNewTab={() => pushTab(
                                                    { title: capitalizeFirstLetter(contextSource.context), icon: '🌍', keepAlive: true },
                                                    () => <ClusterView contextSource={contextSource} />
                                                )}
                                                onSettingsClicked={() => {
                                                    setCurrentSettingsCluster(contextSource);
                                                }}
                                            />
                                        ))
                                    }
                                </div>
                            </div>
                        ))
                    }
                </div>
            </div>
            {
                currentSettingsCluster && (
                    <SettingsDialog open
                        contextSource={currentSettingsCluster}
                        onAfterClose={() => setCurrentSettingsCluster(null)}
                    />
                )
            }
        </>
    );
};

interface ClusterCardWithInfoProps {
    contextSource: KubeContextSource,
    onConnect: () => void,
    onConnectNewTab: () => void,
    onSettingsClicked?: () => void
}

function ClusterCardWithInfo({ contextSource, onConnect, onConnectNewTab, onSettingsClicked }: ClusterCardWithInfoProps) {
    const ref = useRef(null);
    const visible = useIntersectionObserver(ref);

    const { data: version, isPending } = useQuery({
        ...getApiServerGitVersionQuery(contextSource),
        enabled: visible,
    });

    const versionString = isPending ? '…' : version ? version.gitVersion : '—';

    return (
        <ClusterCard ref={ref}
            clusterName={capitalizeFirstLetter(contextSource.context)}
            clusterVersion={versionString}
            onClick={onConnect}
            onAuxClick={onConnectNewTab}
            onSettingsClicked={onSettingsClicked}
        />
    );
}
