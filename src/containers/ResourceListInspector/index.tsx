import { useContext, useEffect, useRef, useState } from "react";
import { Gvk } from "../../model/k8s";

import { EventCallback } from "@tauri-apps/api/event";
import { confirm } from '@tauri-apps/plugin-dialog';
import { deleteResource } from "../../api/deleteResource";
import getResourceYaml from "../../api/getResourceYaml";
import { popupKubernetesResourceMenu } from "../../api/popupKubernetesResourceMenu";
import setDefaultNamespace from "../../api/setDefaultNamespace";
import EmojiHint from "../../components/EmojiHint";
import LogPanel from "../../components/LogPanel";
import { MegaTabContext } from "../../components/MegaTabs/context";
import ResourceList from "../../components/ResourceList";
import { Tab } from "../../components/TabView";
import { TabElement } from "../../components/TabView/hooks";
import HyprkubeTerminal from "../../components/Terminal";
import { DiscoveryResult, useClusterDiscovery } from "../../hooks/useClusterDiscovery";
import useClusterNamespaces from "../../hooks/useClusterNamespaces";
import { KubeContextSource } from "../../hooks/useContextDiscovery";
import useResourceViews from "../../hooks/useResourceViews";
import useResourceWatch, { DisplayableResource } from "../../hooks/useResourceWatch";
import { useTauriEventListener } from "../../hooks/useTauriEventListener";
import ResourceEditor from "../ResourceEditor";
import classes from './styles.module.css';
import { useDefaultNamespace } from "./useDefaultNamespace";

export interface ResourceListInspectorProps {
    gvk: Gvk,
    preSelectedNamespace: string,
    contextSource: KubeContextSource,
    clusterProfile: string,
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
        pushBottomTab,
        preSelectedNamespace,
        onNamespaceChanged = () => undefined
    } = props;

    const availableViews = useResourceViews(contextSource, gvk);
    const [selectedView, setSelectedView] = useState('');
    const { discovery, lastError } = useClusterDiscovery(contextSource.source, contextSource.context);
    const allNamespaces = useClusterNamespaces(contextSource);
    const resourceDefaultNamespace = useDefaultNamespace(clusterProfile, gvk);
    const [selectedNamespace, setSelectedNamespace] = useState(preSelectedNamespace ?? resourceDefaultNamespace);
    const [selectedResources, setSelectedResources] = useState<[string, DisplayableResource][]>([]);
    const [columnDefinitions, resources] = useResourceWatch(contextSource, gvk, selectedView, selectedNamespace);
    const { tabIdentifier } = useContext(MegaTabContext)!;

    const searchbarRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        setSelectedView(availableViews[0]);
    }, [availableViews]);

    const deleteSelectedResources = () => {
        confirm(`This action cannot be reverted. Are you sure?`, { kind: 'warning', title: `Permanently delete ${selectedResources.length} resources?` })
            .then(confirmed => {
                if (!confirmed) return;
                selectedResources.forEach(([, { namespace, name }]) => {
                    deleteResource(contextSource, gvk, namespace, name)
                        .catch(e => alert(JSON.stringify(e)));
                });
            })
            .catch(e => alert(JSON.stringify(e)));
    };

    const saveDefaultNamespace = () => {
        setDefaultNamespace(clusterProfile, gvk, selectedNamespace)
            .catch(e => alert(JSON.stringify(e)));
    };

    const onTriggerEdit: EventCallback<FrontendTriggerResourceEdit> = (event) => {
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
    };

    const onTriggerLogview: EventCallback<FrontendTriggerLogView> = (event) => {
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
    };

    const onTriggerExec: EventCallback<FrontendTriggerExecSession> = (event) => {
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
    };

    const onTriggerPickNamespace: EventCallback<FrontendTriggerPickNamespace> = (event) => {
        setSelectedNamespace(event.payload.namespace);
    };

    useTauriEventListener<FrontendTriggerLogView>('hyprkube:menu:resource:trigger_logs', tabIdentifier.toString(), onTriggerLogview);
    useTauriEventListener<FrontendTriggerResourceEdit>('hyprkube:menu:resource:trigger_edit', tabIdentifier.toString(), onTriggerEdit);
    useTauriEventListener<FrontendTriggerExecSession>('hyprkube:menu:resource:trigger_exec', tabIdentifier.toString(), onTriggerExec);
    useTauriEventListener<FrontendTriggerPickNamespace>('hyprkube:menu:resource:pick_namespace', tabIdentifier.toString(), onTriggerPickNamespace);

    const yamlViewerFactory = () => {
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
    };

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