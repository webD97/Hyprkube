import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';

import { type editor } from 'monaco-editor';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { Editor } from '@monaco-editor/react';
import { confirm } from '@tauri-apps/plugin-dialog';
import { useSearchParams } from 'react-router-dom';
import applyResourceYaml from '../../api/applyResourceYaml';
import { deleteResource } from '../../api/deleteResource';
import getResourceYaml from '../../api/getResourceYaml';
import listClusterProfiles, { ClusterProfile } from '../../api/listClusterProfiles';
import listResourceViews, { type ResourceViewDef } from '../../api/listResourceViews';
import EmojiHint from '../../components/EmojiHint';
import GvkList from '../../components/GvkList';
import ResourceView from '../../components/ResourceView';
import TabView, { Tab } from '../../components/TabView';
import { useTabs } from '../../components/TabView/hooks';
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
    const [selectedNamespace, setSelectedNamespace] = useState('default');
    const [columnTitles, resources] = useResourceWatch(clientId, currentGvk, selectedView, selectedNamespace);
    const [selectedResources, setSelectedResources] = useState<[string, DisplayableResource][]>([]);

    const [tabs, activeTab, pushTab, removeTab, setActiveTab] = useTabs();

    const editorRef = useRef<editor.IStandaloneCodeEditor>(null);

    const searchbarRef = useRef<HTMLDivElement>(null);

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
                        <Tab title={`Show: ${name}`}>
                            {
                                () => (
                                    <>
                                        <button
                                            onClick={() => {
                                                const editor = editorRef.current!;
                                                const data = editor.getValue();

                                                if (!data) return;

                                                applyResourceYaml(clientId, currentGvk!, namespace, name, data)
                                                    .then(newYaml => {
                                                        editor.setValue(newYaml);
                                                    })
                                                    .catch((e) => {
                                                        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
                                                        const {
                                                            status,
                                                            reason,
                                                            message,
                                                        } = JSON.parse(e as string);

                                                        const editorValue = editor.getValue().split('\n').filter(line => !line.startsWith('#')).join('\n');
                                                        editor.setValue(`# ${status} (Reason: ${reason})\n# ${message}\n${editorValue}`);
                                                    });
                                            }}
                                        >
                                            💾 Apply
                                        </button>
                                        <button
                                            onClick={() => {
                                                const editor = editorRef.current!;

                                                getResourceYaml(clientId, gvk, namespace, name)
                                                    .then(yaml => editor.setValue(yaml))
                                                    .catch(e => alert(JSON.stringify(e)));
                                            }}
                                        >
                                            ⭮ Reload
                                        </button>
                                        <Editor
                                            height="600px"
                                            width="100%"
                                            defaultLanguage="yaml"
                                            theme="vs-dark"
                                            options={{
                                                renderWhitespace: "all",
                                            }}
                                            value={yaml}
                                            onMount={(editor) => {
                                                editorRef.current = editor;
                                            }}
                                        />
                                    </>
                                )
                            }
                        </Tab >
                    )
                })
                .catch(e => alert(JSON.stringify(e)));
        }
    }, [clientId, currentGvk, pushTab, resources]);

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
