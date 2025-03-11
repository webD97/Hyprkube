import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';

import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { confirm } from '@tauri-apps/plugin-dialog';
import { useSearchParams } from 'react-router-dom';
import { deleteResource } from '../../api/deleteResource';
import getDefaultNamespace from '../../api/getDefaultNamespace';
import getResourceYaml from '../../api/getResourceYaml';
import listClusterProfiles, { ClusterProfile } from '../../api/listClusterProfiles';
import listResourceViews, { type ResourceViewDef } from '../../api/listResourceViews';
import setDefaultNamespace from '../../api/setDefaultNamespace';
import EmojiHint from '../../components/EmojiHint';
import GvkList from '../../components/GvkList';
import ResourceView from '../../components/ResourceView';
import TabView, { Tab } from '../../components/TabView';
import { useTabs } from '../../components/TabView/hooks';
import ResourceEditor from '../../containers/ResourceEditor';
import { useClusterDiscovery } from '../../hooks/useClusterDiscovery';
import useClusterNamespaces from '../../hooks/useClusterNamespaces';
import useHiddenGvks from '../../hooks/useHiddenGvks';
import usePinnedGvks from '../../hooks/usePinnedGvks';
import useResourceWatch, { DisplayableResource } from '../../hooks/useResourceWatch';
import { Gvk } from '../../model/k8s';
import { createMenuForNormalGvks, createMenuForPinnedGvks, createMenuForResource } from './menus';
import classes from './styles.module.css';

const namespace_gvk = { group: "", version: "v1", kind: "Namespace" };

const ClusterView: React.FC = () => {
    const [searchParams] = useSearchParams();
    const source = searchParams.get('source');
    const context = searchParams.get('context');

    const [availableViews, setAvailableViews] = useState<ResourceViewDef[]>([]);
    const [currentGvk, setCurrentGvk] = useState<Gvk>();
    const [clusterProfiles, setClusterProfiles] = useState<ClusterProfile[]>([]);
    const pinnedGvks = usePinnedGvks(clusterProfiles?.[0]?.[0]);
    const hiddenGvks = useHiddenGvks(clusterProfiles?.[0]?.[0]);
    const [selectedView, setSelectedView] = useState("");
    const { discovery, clientId, lastError } = useClusterDiscovery(source, context);
    const namespaces = useClusterNamespaces(clientId, namespace_gvk);
    const [resourceDefaultNamespace, setResourceDefaultNamespace] = useState('default');
    const [selectedNamespace, setSelectedNamespace] = useState('default');
    const [columnTitles, resources] = useResourceWatch(clientId, currentGvk, selectedView, selectedNamespace);
    const [selectedResources, setSelectedResources] = useState<[string, DisplayableResource][]>([]);

    const [tabs, activeTab, pushTab, removeTab, setActiveTab] = useTabs();

    const searchbarRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (!clusterProfiles[0]?.[0]) return;
        if (!currentGvk) return;

        getDefaultNamespace(clusterProfiles[0][0], currentGvk)
            .then(namespace => {
                setResourceDefaultNamespace(namespace);
                setSelectedNamespace(namespace);
            })
            .catch(e => alert(JSON.stringify(e)))
    }, [clusterProfiles, currentGvk]);

    useEffect(() => {
        listClusterProfiles()
            .then(profiles => {
                setClusterProfiles(profiles);
            })
            .catch(e => alert(JSON.stringify(e)));
    }, []);

    useEffect(() => {
        if (!clientId) return;
        if (!currentGvk) return;

        listResourceViews(clientId, currentGvk)
            .then(views => {
                setAvailableViews(views);

                if (views.length > 0) {
                    setSelectedView(views[0]);
                }
            })
            .catch(e => alert(JSON.stringify(e)));

    }, [currentGvk, clientId]);

    const saveDefaultNamespace = useCallback(() => {
        if (!currentGvk) return;

        setDefaultNamespace(clusterProfiles[0][0], currentGvk, selectedNamespace)
            .catch(e => alert(JSON.stringify(e)));
    }, [clusterProfiles, currentGvk, selectedNamespace]);

    dayjs.extend(relativeTime);

    function findResourcePlural(gvk: Gvk) {
        return discovery.gvks[gvk.group]?.kinds.find(resource => resource.kind === gvk.kind)?.plural;
    }

    function findResourceScope(gvk: Gvk) {
        return discovery.gvks[gvk.group]?.kinds.find(resource => resource.kind === gvk.kind)?.scope;
    }

    const sortedPinnedGvks = useMemo(() => {
        return pinnedGvks.sort((a, b) => {
            if (a.kind < b.kind) {
                return -1;
            }
            if (a.kind > b.kind) {
                return 1;
            }

            return 0;
        });
    }, [pinnedGvks]);

    const yamlViewerFactory = useCallback(() => {
        if (clientId === undefined) {
            return () => undefined;
        }

        return (gvk: Gvk, resourceUID: string) => {
            const { namespace, name } = resources[resourceUID];

            getResourceYaml(clientId, gvk, namespace, name)
                .then((yaml) => {
                    pushTab(
                        <Tab title={`Edit: ${name}`}>
                            {
                                () => (
                                    <ResourceEditor
                                        clientId={clientId}
                                        currentGvk={gvk}
                                        fileContent={yaml}
                                        namespace={namespace}
                                        name={name}
                                    />
                                )
                            }
                        </Tab >
                    )
                })
                .catch(e => alert(JSON.stringify(e)));
        }
    }, [clientId, pushTab, resources]);

    const deleteSelectedResources = useCallback(() => {
        if (!currentGvk) return console.warn('Cannot delete, currentGvk is not set!');
        if (!clientId) return console.warn('Cannot delete, clientId is not set!');

        confirm(`This action cannot be reverted. Are you sure?`, { kind: 'warning', title: `Permanently delete ${selectedResources.length} resources?` })
            .then(confirmed => {
                if (!confirmed) return;
                selectedResources.forEach(([, { namespace, name }]) => {
                    deleteResource(clientId, currentGvk, namespace, name)
                        .catch(e => alert(JSON.stringify(e)));
                });
            })
            .catch(e => alert(JSON.stringify(e)));
    }, [clientId, currentGvk, selectedResources]);

    return (
        <div className={classes.container}>
            <nav>
                <span>{context}</span>
                <h2>Pinned resources</h2>
                {
                    sortedPinnedGvks.length == 0
                        ? null
                        : (
                            <GvkList flat withGroupNames
                                gvks={sortedPinnedGvks}
                                onResourceClicked={(gvk) => setCurrentGvk(gvk)}
                                onGvkContextMenu={(gvk) => createMenuForPinnedGvks({
                                    clusterProfile: clusterProfiles[0][0],
                                    gvk
                                })}
                            />
                        )
                }

                <h2>Builtin resources</h2>
                {
                    Object.values(discovery?.gvks || [])
                        .filter((group) => !group.isCrd)
                        .sort((groupA, groupB) => groupA.name.localeCompare(groupB.name))
                        .map(({ name: groupName, kinds }, idx) => {
                            const gvks = kinds
                                .map(({ kind, version }) => ({ group: groupName, version, kind }))
                                .filter(gvk => (
                                    !hiddenGvks.some(current => (
                                        current.group === gvk.group &&
                                        current.version === gvk.version &&
                                        current.kind === gvk.kind
                                    ))
                                ));

                            // Don't show groups where everything is hidden
                            if (gvks.length === 0) return;

                            return (
                                <GvkList key={idx}
                                    gvks={gvks}
                                    onResourceClicked={(gvk) => setCurrentGvk(gvk)}
                                    onGvkContextMenu={(gvk) => createMenuForNormalGvks({
                                        clusterProfile: clusterProfiles[0][0],
                                        gvk
                                    })}
                                />
                            );
                        })
                }

                <h2>Custom resources</h2>
                {
                    Object.values(discovery?.gvks || [])
                        .filter((group) => group.isCrd)
                        .sort((groupA, groupB) => groupA.name.localeCompare(groupB.name))
                        .map(({ name: groupName, kinds }, idx) => {
                            const gvks = kinds
                                .map(({ kind, version }) => ({ group: groupName, version, kind }))
                                .filter(gvk => (
                                    !hiddenGvks.some(current => (
                                        current.group === gvk.group &&
                                        current.version === gvk.version &&
                                        current.kind === gvk.kind
                                    ))
                                ));

                            // Don't show groups where everything is hidden
                            if (gvks.length === 0) return;

                            return (
                                <GvkList key={idx}
                                    gvks={gvks}
                                    onResourceClicked={(gvk) => setCurrentGvk(gvk)}
                                    onGvkContextMenu={(gvk) => createMenuForNormalGvks({
                                        clusterProfile: clusterProfiles[0][0],
                                        gvk
                                    })}
                                />
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
                    lastError !== undefined
                        ? <EmojiHint emoji="💩"><span style={{ color: 'red' }}>{lastError}</span></EmojiHint>
                        : currentGvk === undefined
                            ? <EmojiHint emoji="👈">Select a resource to get started.</EmojiHint>
                            : (
                                <>
                                    <div className={classes.topBar}>
                                        <h2>{findResourcePlural(currentGvk)}</h2>
                                        <select value={selectedView} onChange={(e) => setSelectedView(e.target.value)}>
                                            {
                                                availableViews.map(view => (
                                                    <option key={view}>{view}</option>
                                                ))
                                            }
                                        </select>
                                        {
                                            findResourceScope(currentGvk) === 'cluster'
                                                ? null
                                                : (
                                                    <>
                                                        <select value={selectedNamespace} onChange={(e) => setSelectedNamespace(e.target.value)}>
                                                            <option label="(All namespaces)"></option>
                                                            {
                                                                Object.values(namespaces).map(namespace => (
                                                                    <option key={namespace} value={namespace}>
                                                                        {namespace}
                                                                        {
                                                                            resourceDefaultNamespace === namespace
                                                                                ? ' ⭐'
                                                                                : ''
                                                                        }
                                                                    </option>
                                                                ))
                                                            }
                                                        </select>
                                                        {
                                                            resourceDefaultNamespace !== selectedNamespace
                                                                ? <button title="Save as custom default namespace" onClick={saveDefaultNamespace}>💾 Save as default</button>
                                                                : null
                                                        }
                                                    </>
                                                )
                                        }
                                        {
                                            selectedResources.length < 1
                                                ? null
                                                : <button onClick={deleteSelectedResources}>🗑️ Delete {selectedResources.length}</button>
                                        }
                                        <div ref={searchbarRef}></div>
                                    </div>
                                    <ResourceView
                                        resourceNamePlural={findResourcePlural(currentGvk)}
                                        gvk={currentGvk}
                                        namespace={selectedNamespace}
                                        columnTitles={columnTitles || []}
                                        resourceData={resources}
                                        onResourceClicked={yamlViewerFactory()}
                                        searchbarPortal={searchbarRef}
                                        onResourceContextMenu={(gvk, resourceUID) => {
                                            const { namespace, name } = resources[resourceUID];

                                            return createMenuForResource({
                                                clientId: clientId!, gvk, namespace, name, pushTab,
                                                onShowYaml: () => yamlViewerFactory()(gvk, resourceUID),
                                                onSelectNamespace: (namespace) => {
                                                    setSelectedNamespace(namespace)
                                                },
                                            });
                                        }}
                                        onSelectionChanged={setSelectedResources}
                                    />
                                </>
                            )
                }
            </section>
        </div>
    )
}

export default ClusterView;
