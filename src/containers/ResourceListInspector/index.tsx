import { use, useCallback, useEffect, useEffectEvent, useRef, useState } from "react";
import { Gvk } from "../../model/k8s";

import { useQuery } from "@tanstack/react-query";
import { EventCallback } from "@tauri-apps/api/event";
import { confirm } from '@tauri-apps/plugin-dialog';
import { Button, Select, Switch } from "antd";
import { deleteResource } from "../../api/deleteResource";
import getDefaultNamespace from "../../api/getDefaultNamespace";
import getResourceYaml from "../../api/getResourceYaml";
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
import listResourcePresentationsQuery from "../../queries/listResourcePresentations";
import ResourceEditor from "../ResourceEditor";
import classes from './styles.module.css';

export interface ResourceListInspectorProps {
    gvk: Gvk,
    preSelectedNamespace?: string,
    namespaceLocked: boolean,
    contextSource: KubeContextSource,
    clusterProfile: string,
    discovery: DiscoveryResult,
    pushBottomTab: (tab: TabElement) => void,
    onNamespaceChanged?: (namespace: string) => void,
    onNamespaceLockChanged?: (locked: boolean) => void,
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
        namespaceLocked,
        onNamespaceChanged = () => undefined,
        onNamespaceLockChanged = () => undefined
    } = props;

    const [selectedPresentation, setSelectedPresentation] = useState("");
    const availablePresentations = useQuery({
        ...listResourcePresentationsQuery(contextSource, gvk),
        initialData: [],
    });
    const allNamespaces = useClusterNamespaces(contextSource);
    // The pre-selected namespace only matters on mount; we're remounted whenever the GVK changes
    const [initialNamespace] = useState(preSelectedNamespace);
    // undefined until we know which namespace to show, so we don't start watching the wrong one
    const [selectedNamespace, setSelectedNamespace] = useState(initialNamespace);
    const [resourceDefaultNamespace, setResourceDefaultNamespace] = useState<string>();
    const [selectedResources, setSelectedResources] = useState<[string, DisplayableResource][]>([]);
    const [columnDefinitions, resources] = useResourceWatch(contextSource, gvk, selectedPresentation, selectedNamespace);
    const { tabIdentifier } = use(MegaTabContext)!;

    const searchbarRef = useRef<HTMLDivElement>(null);

    if (availablePresentations.data.length > 0 && !availablePresentations.data.includes(selectedPresentation)) {
        setSelectedPresentation(availablePresentations.data[0]);
    }

    const resourceScope = findResourceScope(discovery, gvk);
    const resourceNamePlural = findResourcePlural(discovery, gvk);

    useEffect(() => {
        let cancelled = false;

        getDefaultNamespace(clusterProfile, gvk)
            .then(namespace => {
                if (cancelled) return;
                setResourceDefaultNamespace(namespace);
                setSelectedNamespace(selected => selected ?? namespace);
            })
            .catch(e => alert(JSON.stringify(e)));

        return () => {
            cancelled = true;
        };
    }, [clusterProfile, gvk]);

    const notifyNamespaceChanged = useEffectEvent((namespace: string) => {
        // Cluster-scoped resources have no namespace, so keep whatever the parent has
        if (resourceScope === 'cluster') return;
        onNamespaceChanged(namespace);
    });

    useEffect(() => {
        if (selectedNamespace === undefined) return;
        notifyNamespaceChanged(selectedNamespace);
    }, [selectedNamespace]);

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
        if (selectedNamespace === undefined) return;

        setDefaultNamespace(clusterProfile, gvk, selectedNamespace)
            .then(() => setResourceDefaultNamespace(selectedNamespace))
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

    const markDefault = (label: string, namespace: string) => namespace === resourceDefaultNamespace ? label + ' ⭐' : label;

    return (
        <div className={classes.container}>
            <div className={classes.topBar}>
                <h2>{resourceNamePlural}</h2>
                <Select style={{ minWidth: '150px' }}
                    options={availablePresentations.data.map(v => ({ label: v, value: v }))}
                    value={selectedPresentation}
                    onChange={(value) => setSelectedPresentation(value)}
                />
                {
                    resourceScope === 'cluster'
                        ? null
                        : (
                            <>
                                <Select style={{ minWidth: '200px' }}
                                    options={[
                                        { label: markDefault('(All namespaces)', ''), value: '' },
                                        ...allNamespaces.map(n => ({ label: markDefault(n, n), value: n }))
                                    ]}
                                    value={selectedNamespace}
                                    onChange={(value) => setSelectedNamespace(value)}
                                    showSearch={{ optionFilterProp: 'label' }}
                                    popupMatchSelectWidth={false}
                                    listHeight={512}
                                />
                                <Switch
                                    checked={namespaceLocked}
                                    onChange={onNamespaceLockChanged}
                                    checkedChildren="🔒"
                                    unCheckedChildren="🔓"
                                    aria-label="Lock namespace"
                                    title={
                                        namespaceLocked
                                            ? 'Namespace locked: stays in this namespace when switching resources'
                                            : 'Namespace unlocked: switching resources uses their default namespace'
                                    }
                                />
                                {
                                    resourceDefaultNamespace !== undefined && selectedNamespace !== undefined && resourceDefaultNamespace !== selectedNamespace
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
                    contextSource={contextSource}
                    resourceNamePlural={resourceNamePlural}
                    gvk={gvk}
                    namespace={selectedNamespace}
                    columnDefinitions={columnDefinitions || []}
                    resourceData={resources}
                    onResourceClicked={yamlViewerFactory()}
                    searchbarPortal={searchbarRef}
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