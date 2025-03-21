import { useCallback, useEffect, useRef, useState } from "react";
import { Gvk } from "../../model/k8s";

import { confirm } from '@tauri-apps/plugin-dialog';
import { deleteResource } from "../../api/deleteResource";
import getDefaultNamespace from "../../api/getDefaultNamespace";
import getResourceYaml from "../../api/getResourceYaml";
import listResourceViews, { ResourceViewDef } from "../../api/listResourceViews";
import setDefaultNamespace from "../../api/setDefaultNamespace";
import EmojiHint from "../../components/EmojiHint";
import ResourceList from "../../components/ResourceList";
import { Tab } from "../../components/TabView";
import { TabElement } from "../../components/TabView/hooks";
import { DiscoveryResult, useClusterDiscovery } from "../../hooks/useClusterDiscovery";
import useClusterNamespaces from "../../hooks/useClusterNamespaces";
import { KubeContextSource } from "../../hooks/useContextDiscovery";
import useResourceWatch, { DisplayableResource } from "../../hooks/useResourceWatch";
import ResourceEditor from "../ResourceEditor";
import { createMenuForResource } from "./menus";
import classes from './styles.module.css';

export interface ResourceListInspectorProps {
    gvk: Gvk,
    preSelectedNamespace: string,
    contextSource: KubeContextSource,
    clusterProfile: string,
    pushBottomTab: (tab: TabElement) => void,
    onNamespaceChanged?: (namespace: string) => void,
}

const ResourceListInspector: React.FC<ResourceListInspectorProps> = (props) => {
    const {
        gvk,
        contextSource,
        clusterProfile,
        pushBottomTab,
        preSelectedNamespace,
        onNamespaceChanged = () => undefined
    } = props;

    const [availableViews, setAvailableViews] = useState<ResourceViewDef[]>([]);
    const [selectedView, setSelectedView] = useState("");
    const { discovery, clientId, lastError } = useClusterDiscovery(contextSource.source, contextSource.context);
    const allNamespaces = useClusterNamespaces(clientId);
    const [selectedNamespace, setSelectedNamespace] = useState(preSelectedNamespace);
    const [resourceDefaultNamespace, setResourceDefaultNamespace] = useState('default');
    const [selectedResources, setSelectedResources] = useState<[string, DisplayableResource][]>([]);
    const [columnTitles, resources] = useResourceWatch(clientId, gvk, selectedView, selectedNamespace);

    const searchbarRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (!clientId) return;

        listResourceViews(clientId, gvk)
            .then(views => {
                setAvailableViews(views);

                if (views.length > 0) {
                    setSelectedView(views[0]);
                }
            })
            .catch(e => alert(JSON.stringify(e)));

    }, [clientId, gvk]);

    useEffect(() => {
        if (preSelectedNamespace) return;

        getDefaultNamespace(clusterProfile, gvk)
            .then(namespace => {
                setResourceDefaultNamespace(namespace);
                setSelectedNamespace(namespace);
                onNamespaceChanged(namespace);
            })
            .catch(e => alert(JSON.stringify(e)))
    }, [clusterProfile, gvk, onNamespaceChanged, preSelectedNamespace]);

    const deleteSelectedResources = useCallback(() => {
        if (!clientId) return console.warn('Cannot delete, clientId is not set!');

        confirm(`This action cannot be reverted. Are you sure?`, { kind: 'warning', title: `Permanently delete ${selectedResources.length} resources?` })
            .then(confirmed => {
                if (!confirmed) return;
                selectedResources.forEach(([, { namespace, name }]) => {
                    deleteResource(clientId, gvk, namespace, name)
                        .catch(e => alert(JSON.stringify(e)));
                });
            })
            .catch(e => alert(JSON.stringify(e)));
    }, [clientId, gvk, selectedResources]);

    const saveDefaultNamespace = useCallback(() => {
        setDefaultNamespace(clusterProfile, gvk, selectedNamespace)
            .catch(e => alert(JSON.stringify(e)));
    }, [clusterProfile, gvk, selectedNamespace]);

    const yamlViewerFactory = useCallback(() => {
        if (clientId === undefined) {
            return () => undefined;
        }

        return (gvk: Gvk, resourceUID: string) => {
            const { namespace, name } = resources[resourceUID];

            getResourceYaml(clientId, gvk, namespace, name)
                .then((yaml) => {
                    pushBottomTab(
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
    }, [clientId, pushBottomTab, resources]);

    const resourceScope = findResourceScope(discovery, gvk);
    const resourceNamePlural = findResourcePlural(discovery, gvk);

    if (lastError !== undefined) {
        return <EmojiHint emoji="💩"><span style={{ color: 'red' }}>{lastError}</span></EmojiHint>
    }

    return (
        <div className={classes.container}>
            <div className={classes.topBar}>
                <h2>{resourceNamePlural}</h2>
                <select value={selectedView} onChange={(e) => setSelectedView(e.target.value)}>
                    {
                        availableViews.map(view => (
                            <option key={view}>{view}</option>
                        ))
                    }
                </select>
                {
                    resourceScope === 'cluster'
                        ? null
                        : (
                            <>
                                <select value={selectedNamespace} onChange={(e) => {
                                    setSelectedNamespace(e.target.value);
                                    onNamespaceChanged(e.target.value);
                                }}>
                                    <option label="(All namespaces)"></option>
                                    {
                                        Object.values(allNamespaces).map(namespace => (
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
            <div className={classes.tableArea}>
                <ResourceList
                    resourceNamePlural={resourceNamePlural}
                    gvk={gvk}
                    namespace={selectedNamespace}
                    columnTitles={columnTitles || []}
                    resourceData={resources}
                    onResourceClicked={yamlViewerFactory()}
                    searchbarPortal={searchbarRef}
                    onResourceContextMenu={(gvk, resourceUID) => {
                        const { namespace, name } = resources[resourceUID];

                        return createMenuForResource({
                            clientId: clientId!, gvk, namespace, name, pushTab: pushBottomTab,
                            onShowYaml: () => yamlViewerFactory()(gvk, resourceUID),
                            onSelectNamespace: (namespace) => {
                                setSelectedNamespace(namespace)
                            },
                        });
                    }}
                    onSelectionChanged={setSelectedResources}
                />
            </div>
        </div>
    );
};

function findResourcePlural(discovery: DiscoveryResult, gvk: Gvk): string {
    return discovery.gvks[gvk.group]?.kinds.find(resource => resource.kind === gvk.kind)?.plural || gvk.kind + 's';
}

function findResourceScope(discovery: DiscoveryResult, gvk: Gvk) {
    return discovery.gvks[gvk.group]?.kinds.find(resource => resource.kind === gvk.kind)?.scope || 'namespaced';
}

export default ResourceListInspector;