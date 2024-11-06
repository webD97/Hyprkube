import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';

import { useEffect, useState } from 'react';

import EmojiHint from '../../components/EmojiHint';
import GvkList from '../../components/GvkList';
import LogPanel from '../../components/LogPanel';
import ResourceView from '../../components/ResourceView';
import TabView, { Tab } from '../../components/TabView';
import { useTabs } from '../../components/TabView/hooks';
import { useClusterDiscovery } from '../../hooks/useClusterDiscovery';
import useResourceWatch from '../../hooks/useResourceWatch';
import { Gvk, NamespaceAndName } from '../../model/k8s';
import classes from './styles.module.css';
import { useSearchParams } from 'react-router-dom';
import useClusterNamespaces from '../../hooks/useClusterNamespaces';
import { deleteResource } from '../../api/deleteResource';

const namespace_gvk = { group: "", version: "v1", kind: "Namespace" };

const defaultPinnedGvks: Gvk[] = [
    { group: '', version: 'v1', kind: 'Node' },
    { group: '', version: 'v1', kind: 'Namespace' },
    { group: '', version: 'v1', kind: 'Pod' },
    { group: 'apps', version: 'v1', kind: 'Deployment' },
    { group: 'apps', version: 'v1', kind: 'StatefulSet' },
    { group: 'batch', version: 'v1', kind: 'CronJob' },
    { group: 'batch', version: 'v1', kind: 'Job' },
    { group: '', version: 'v1', kind: 'ConfigMap' },
    { group: '', version: 'v1', kind: 'Secret' },
    { group: '', version: 'v1', kind: 'Service' },
    { group: 'networking.k8s.io', version: 'v1', kind: 'Ingress' },
    { group: '', version: 'v1', kind: 'PersistentVolumeClaim' },
    { group: '', version: 'v1', kind: 'PersistentVolume' },
];

const ClusterView: React.FC = () => {
    const [searchParams] = useSearchParams();
    const source = searchParams.get('source');
    const context = searchParams.get('context');

    const [currentGvk, setCurrentGvk] = useState<Gvk>();
    const [pinnedGvks, setPinnedGvks] = useState<Gvk[]>(defaultPinnedGvks);
    const [selectedResource, setSelectedResource] = useState<NamespaceAndName>({ namespace: '', name: '' });
    const [selectedView, setSelectedView] = useState("");
    const { discovery, clientId, lastError, loading } = useClusterDiscovery(source, context);
    const namespaces = useClusterNamespaces(clientId, namespace_gvk);
    const [selectedNamespace, setSelectedNamespace] = useState('default');
    const [columnTitles, resources] = useResourceWatch(clientId, currentGvk, selectedView, selectedNamespace);

    const [tabs, activeTab, pushTab, removeTab, setActiveTab] = useTabs();

    useEffect(() => {
        if (!currentGvk) return;

        const availableViews = discovery?.gvks[currentGvk.group].kinds.find(k => k.kind === currentGvk.kind)?.views || [];

        if (availableViews?.length < 1) return;

        setSelectedView(availableViews[0]);
    }, [selectedResource, currentGvk, discovery?.gvks]);

    dayjs.extend(relativeTime);

    function findResourcePlural(gvk: Gvk) {
        return discovery.gvks[gvk.group]?.kinds.find(resource => resource.kind === gvk.kind)?.plural;
    }

    function findResourceScope(gvk: Gvk) {
        return discovery.gvks[gvk.group]?.kinds.find(resource => resource.kind === gvk.kind)?.scope;
    }

    return (
        <div className={classes.container}>
            <nav>
                <span>{context}</span>
                <h2>Pinned resources</h2>
                {
                    pinnedGvks.length == 0
                        ? null
                        : (
                            <GvkList withGroupName
                                gvks={pinnedGvks}
                                onResourceClicked={(gvk) => setCurrentGvk(gvk)}
                                onPinButtonClicked={({ group, kind }) => setPinnedGvks(currentlyPinned => currentlyPinned.filter(clickedGvk => clickedGvk.group !== group || clickedGvk.kind !== kind))}
                            />
                        )
                }

                <h2>Builtin resources</h2>
                {
                    Object.values(discovery?.gvks || [])
                        .filter((group) => !group.isCrd)
                        .sort((groupA, groupB) => groupA.name.localeCompare(groupB.name))
                        .map(({ name: groupName, kinds }, idx) => {
                            const gvks = kinds.map(({ kind, version }) => ({ group: groupName, version, kind }));

                            return (
                                <details key={idx}>
                                    <summary>{groupName ? groupName : 'core'}</summary>
                                    <GvkList className={classes.gvkListIndented}
                                        gvks={gvks}
                                        onResourceClicked={(gvk) => setCurrentGvk(gvk)}
                                        onPinButtonClicked={(gvk) => setPinnedGvks(currentlyPinned => [...currentlyPinned, gvk])}
                                    />
                                </details>
                            );
                        })
                }

                <h2>Custom resources</h2>
                {
                    Object.values(discovery?.gvks || [])
                        .filter((group) => group.isCrd)
                        .sort((groupA, groupB) => groupA.name.localeCompare(groupB.name))
                        .map(({ name: groupName, kinds }) => {
                            const gvks = kinds.map(({ kind, version }) => ({ group: groupName, version, kind }));

                            return (
                                <details key={groupName}>
                                    <summary>{groupName ? groupName : 'core'}</summary>
                                    <GvkList className={classes.gvkListIndented}
                                        gvks={gvks}
                                        onResourceClicked={(gvk) => setCurrentGvk(gvk)}
                                        onPinButtonClicked={(gvk) => setPinnedGvks(currentlyPinned => [...currentlyPinned, gvk])}
                                    />
                                </details>
                            );
                        })
                }
            </nav>
            <section className={classes.bottomPanel}>
                <TabView
                    activeTab={activeTab}
                    onCloseClicked={(idx) => removeTab(idx)}
                    setActiveTab={setActiveTab}
                >
                    {tabs}
                </TabView>
            </section>
            <section className={classes.mainArea}>
                {
                    loading
                        ? <EmojiHint emoji="⏳">Loading...</EmojiHint>
                        : lastError !== undefined
                            ? <EmojiHint emoji="💩"><span style={{ color: 'red' }}>{lastError}</span></EmojiHint>
                            : currentGvk === undefined
                                ? <EmojiHint emoji="🔍">Select a resource to get started.</EmojiHint>
                                : (
                                    <>
                                        <div className={classes.topBar}>
                                            <h2>{findResourcePlural(currentGvk)}</h2>
                                            <select value={selectedView} onChange={(e) => setSelectedView(e.target.value)}>
                                                {
                                                    discovery?.gvks[currentGvk.group].kinds.find(v => v.kind === currentGvk.kind)?.views.map(view => (
                                                        <option key={view}>{view}</option>
                                                    ))
                                                }
                                            </select>
                                            {
                                                findResourceScope(currentGvk) === 'cluster'
                                                    ? null
                                                    : (
                                                        <select value={selectedNamespace} onChange={(e) => setSelectedNamespace(e.target.value)}>
                                                            <option label="(All namespaces)"></option>
                                                            {
                                                                Object.values(namespaces).map(namespace => (
                                                                    <option key={namespace}>{namespace}</option>
                                                                ))
                                                            }
                                                        </select>
                                                    )
                                            }
                                        </div>
                                        <ResourceView
                                            resourceNamePlural={findResourcePlural(currentGvk)}
                                            namespace={selectedNamespace}
                                            columnTitles={columnTitles || []}
                                            resourceData={resources}
                                            onDeleteClicked={(uid) => {
                                                const { namespace, name } = resources[uid];
                                                if (window.confirm(`Do you really want to delete ${name}?`)) {
                                                    deleteResource(clientId!, currentGvk, namespace, name);
                                                }
                                            }}
                                            onResourceClicked={(uid) => {
                                                setSelectedResource(resources[uid]);

                                                if (currentGvk.kind === "Pod") {
                                                    pushTab(
                                                        <Tab title={resources[uid].name}>
                                                            {
                                                                () => (
                                                                    <LogPanel
                                                                        kubernetesClientId={clientId}
                                                                        namespace={resources[uid].namespace}
                                                                        name={resources[uid].name}
                                                                    />
                                                                )
                                                            }
                                                        </Tab>
                                                    );
                                                }
                                            }}
                                        />
                                    </>
                                )
                }
            </section>
        </div>
    )
}

export default ClusterView;
