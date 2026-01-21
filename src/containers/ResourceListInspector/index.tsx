import { use, useCallback, useEffect, useRef, useState } from "react";
import { Gvk } from "../../model/k8s";

import { EventCallback } from "@tauri-apps/api/event";
import { confirm } from '@tauri-apps/plugin-dialog';
import { Button, Select } from "antd";
import { deleteResource } from "../../api/deleteResource";
import getDefaultNamespace from "../../api/getDefaultNamespace";
import getResourceYaml from "../../api/getResourceYaml";
import listResourceViews, { ResourceViewDef } from "../../api/listResourceViews";
import { popupKubernetesResourceMenu } from "../../api/popupKubernetesResourceMenu";
import setDefaultNamespace from "../../api/setDefaultNamespace";
import LogPanel from "../../components/LogPanel";
import ResourceList from "../../components/ResourceList";
import { Tab } from "../../components/TabView";
import { TabElement } from "../../components/TabView/hooks";
import HyprkubeTerminal from "../../components/Terminal";
import { MegaTabContext } from "../../contexts/MegaTab";
import { DiscoveryResult } from "../../hooks/useClusterDiscovery";
import useClusterNamespaces from "../../hooks/useClusterNamespaces";
import { KubeContextSource } from "../../hooks/useContextDiscovery";
import useResourceWatch, { DisplayableResource } from "../../hooks/useResourceWatch";
import { useTauriEventListener } from "../../hooks/useTauriEventListener";
import ResourceEditor from "../ResourceEditor";
import classes from './styles.module.css';

export interface ResourceListInspectorProps {
    gvk: Gvk,
    preSelectedNamespace: string,
    contextSource: KubeContextSource,
    clusterProfile: string,
    discovery: DiscoveryResult,
    pushBottomTab: (tab: TabElement) => void,
    onNamespaceChanged?: (namespace: string) => void,
}

type FrontendTriggerResourceEdit = { tabId: string, gvk: Gvk, namespace: string, name: string };
type FrontendTriggerLogView = { tabId: string, namespace: string, name: string, container: string };
type FrontendTriggerExecSession = { tabId: string, namespace: string, name: string, container: string };
type FrontendTriggerPickNamespace = { tabId: string, namespace: string };

const ResourceListInspector: React.FC<ResourceListInspectorProps> = (props) => {
    const {
        gvk,
        contextSource,
        clusterProfile,
        discovery,
        pushBottomTab,
        preSelectedNamespace,
        onNamespaceChanged = () => undefined
    } = props;

    const [availableViews, setAvailableViews] = useState<ResourceViewDef[]>([]);
    const [selectedView, setSelectedView] = useState("");
    // const { discovery, lastError } = useClusterDiscovery(contextSource.source, contextSource.context);
    const allNamespaces = useClusterNamespaces(contextSource);
    const [selectedNamespace, setSelectedNamespace] = useState(preSelectedNamespace);
    const [resourceDefaultNamespace, setResourceDefaultNamespace] = useState('default');
    const [selectedResources, setSelectedResources] = useState<[string, DisplayableResource][]>([]);
    const [columnDefinitions, resources] = useResourceWatch(contextSource, gvk, selectedView, selectedNamespace);
    const { tabIdentifier } = use(MegaTabContext)!;

    const searchbarRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        listResourceViews(contextSource, gvk)
            .then(views => {
                setAvailableViews(views);

                if (views.length > 0) {
                    setSelectedView(views[0]);
                }
            })
            .catch(e => alert(JSON.stringify(e)));

    }, [contextSource, gvk]);

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
        confirm(`This action cannot be reverted. Are you sure?`, { kind: 'warning', title: `Permanently delete ${selectedResources.length} resources?` })
            .then(confirmed => {
                if (!confirmed) return;
                selectedResources.forEach(([, { namespace, name }]) => {
                    deleteResource(contextSource, gvk, namespace, name)
                        .catch(e => alert(JSON.stringify(e)));
                });
            })
            .catch(e => alert(JSON.stringify(e)));
    }, [contextSource, gvk, selectedResources]);

    const saveDefaultNamespace = useCallback(() => {
        setDefaultNamespace(clusterProfile, gvk, selectedNamespace)
            .catch(e => alert(JSON.stringify(e)));
    }, [clusterProfile, gvk, selectedNamespace]);

    const onTriggerEdit = useCallback<EventCallback<FrontendTriggerResourceEdit>>((event) => {
        const { gvk, namespace, name } = event.payload;

        getResourceYaml(contextSource, gvk, namespace, name)
            .then((yaml) => {
                pushBottomTab(
                    <Tab title={`Edit: ${name}`}>
                        {
                            () => (
                                <ResourceEditor
                                    contextSource={contextSource}
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
    }, [contextSource, pushBottomTab]);

    const onTriggerLogview = useCallback<EventCallback<FrontendTriggerLogView>>((event) => {
        const { container, namespace, name } = event.payload;

        pushBottomTab(
            <Tab title={container} >
                {
                    () => (
                        <LogPanel
                            contextSource={contextSource}
                            namespace={namespace}
                            name={name}
                            container={container}
                        />
                    )
                }
            </Tab>
        );
    }, [contextSource, pushBottomTab]);

    const onTriggerExec = useCallback<EventCallback<FrontendTriggerExecSession>>((event) => {
        const { container, namespace, name } = event.payload;

        pushBottomTab(
            <Tab title={`Shell (${name})`}>
                {
                    () => (
                        <HyprkubeTerminal
                            contextSource={contextSource}
                            podName={name}
                            podNamespace={namespace}
                            container={container}
                        />
                    )
                }
            </Tab>
        );
    }, [contextSource, pushBottomTab]);

    const onTriggerPickNamespace = useCallback<EventCallback<FrontendTriggerPickNamespace>>((event) => {
        setSelectedNamespace(event.payload.namespace);
    }, []);

    useTauriEventListener<FrontendTriggerLogView>('hyprkube:menu:resource:trigger_logs', tabIdentifier.toString(), onTriggerLogview);
    useTauriEventListener<FrontendTriggerResourceEdit>('hyprkube:menu:resource:trigger_edit', tabIdentifier.toString(), onTriggerEdit);
    useTauriEventListener<FrontendTriggerExecSession>('hyprkube:menu:resource:trigger_exec', tabIdentifier.toString(), onTriggerExec);
    useTauriEventListener<FrontendTriggerPickNamespace>('hyprkube:menu:resource:pick_namespace', tabIdentifier.toString(), onTriggerPickNamespace);

    const yamlViewerFactory = useCallback(() => {
        return (gvk: Gvk, resourceUID: string) => {
            const { namespace, name } = resources[resourceUID];

            getResourceYaml(contextSource, gvk, namespace, name)
                .then((yaml) => {
                    pushBottomTab(
                        <Tab title={`Edit: ${name}`}>
                            {
                                () => (
                                    <ResourceEditor
                                        contextSource={contextSource}
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
    }, [contextSource, pushBottomTab, resources]);

    const resourceScope = findResourceScope(discovery, gvk);
    const resourceNamePlural = findResourcePlural(discovery, gvk);

    return (
        <div className={classes.container}>
            <div className={classes.topBar}>
                <h2>{resourceNamePlural}</h2>
                <Select
                    options={availableViews.map(v => ({ label: v, value: v }))}
                    value={selectedView}
                    onChange={(value) => setSelectedView(value)}
                />
                {
                    resourceScope === 'cluster'
                        ? null
                        : (
                            <>
                                <Select style={{ minWidth: '200px' }}
                                    options={[
                                        { label: '(All namespaces)', value: '' },
                                        ...allNamespaces.map(n => ({ label: n === resourceDefaultNamespace ? n + ' ⭐' : n, value: n }))
                                    ]}
                                    value={selectedNamespace}
                                    onChange={(value) => setSelectedNamespace(value)}
                                    showSearch={{ optionFilterProp: 'label' }}
                                    popupMatchSelectWidth={false}
                                    listHeight={512}
                                />
                                {
                                    resourceDefaultNamespace !== selectedNamespace
                                        ? <Button icon="💾" title="Save as custom default namespace" onClick={saveDefaultNamespace}>Save as default</Button>
                                        : null
                                }
                            </>
                        )
                }
                {
                    selectedResources.length < 1
                        ? null
                        : <Button icon="🗑️" onClick={deleteSelectedResources}> Delete {selectedResources.length}</Button>
                }
                <div ref={searchbarRef}></div>
            </div>
            <div className={classes.tableArea}>
                <ResourceList
                    resourceNamePlural={resourceNamePlural}
                    gvk={gvk}
                    namespace={selectedNamespace}
                    columnDefinitions={columnDefinitions || []}
                    resourceData={resources}
                    onResourceClicked={yamlViewerFactory()}
                    searchbarPortal={searchbarRef}
                    onResourceContextMenu={(gvk, resourceUID, position) => {
                        const { namespace, name } = resources[resourceUID];

                        popupKubernetesResourceMenu(contextSource, tabIdentifier.toString(), namespace, name, gvk, position)
                            .catch(e => console.log(e))
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