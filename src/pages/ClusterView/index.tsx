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
import { Gvk } from '../../model/k8s';
import classes from './styles.module.css';
import { useSearchParams } from 'react-router-dom';
import useClusterNamespaces from '../../hooks/useClusterNamespaces';
import { deleteResource } from '../../api/deleteResource';
import listResourceViews, { type ResourceViewDef } from '../../api/listResourceViews';
import { confirm } from '@tauri-apps/plugin-dialog';
import { Menu, MenuItem, PredefinedMenuItem, Submenu } from '@tauri-apps/api/menu';

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

    const [availableViews, setAvailableViews] = useState<ResourceViewDef[]>([]);
    const [currentGvk, setCurrentGvk] = useState<Gvk>();
    const [pinnedGvks, setPinnedGvks] = useState<Gvk[]>(defaultPinnedGvks);
    const [selectedView, setSelectedView] = useState("");
    const { discovery, clientId, lastError, loading } = useClusterDiscovery(source, context);
    const namespaces = useClusterNamespaces(clientId, namespace_gvk);
    const [selectedNamespace, setSelectedNamespace] = useState('default');
    const [columnTitles, resources] = useResourceWatch(clientId, currentGvk, selectedView, selectedNamespace);

    const [tabs, activeTab, pushTab, removeTab, setActiveTab] = useTabs();

    useEffect(() => {
        if (!clientId) return;
        if (!currentGvk) return;

        listResourceViews(clientId, currentGvk)
            .then(views => {
                setAvailableViews(views);

                if (views.length > 0) {
                    setSelectedView(views[0]);
                }
            });

    }, [currentGvk, clientId]);

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
                                        </div>
                                        <ResourceView
                                            resourceNamePlural={findResourcePlural(currentGvk)}
                                            namespace={selectedNamespace}
                                            columnTitles={columnTitles || []}
                                            resourceData={resources}
                                            onResourceContextMenu={async (resourceUID: string) => {
                                                const itemPromises: Promise<MenuItem | PredefinedMenuItem>[] = [
                                                    MenuItem.new({
                                                        text: 'Show YAML',
                                                        enabled: false,
                                                    }),
                                                    MenuItem.new({
                                                        text: 'Copy YAML',
                                                        enabled: false,
                                                    }),
                                                    MenuItem.new({
                                                        text: 'Delete resource',
                                                        action: async () => {
                                                            const { namespace, name } = resources[resourceUID];

                                                            const confirmed = await confirm(`This action cannot be reverted. Are you sure?`, { kind: 'warning', title: `Permanently delete resource?` });

                                                            if (confirmed) {
                                                                deleteResource(clientId!, currentGvk, namespace, name);
                                                            }
                                                        }
                                                    }),
                                                    PredefinedMenuItem.new({ item: 'Separator' }),
                                                ];

                                                if (currentGvk.kind === "Pod") {
                                                    const logItems: Promise<MenuItem>[] = [];
                                                    const attachItems: Promise<MenuItem>[] = [];

                                                    logItems.push(
                                                        MenuItem.new({
                                                            text: 'Container 0',
                                                            action: () => {
                                                                pushTab(
                                                                    <Tab title={resources[resourceUID].name}>
                                                                        {
                                                                            () => (
                                                                                <LogPanel
                                                                                    kubernetesClientId={clientId}
                                                                                    namespace={resources[resourceUID].namespace}
                                                                                    name={resources[resourceUID].name}
                                                                                />
                                                                            )
                                                                        }
                                                                    </Tab>
                                                                )
                                                            }
                                                        })
                                                    );

                                                    attachItems.push(
                                                        MenuItem.new({
                                                            text: 'Container 0',
                                                            enabled: false
                                                        })
                                                    );

                                                    const logsSubmenu = Submenu.new({
                                                        text: 'Show logs',
                                                        items: await Promise.all(logItems)
                                                    })

                                                    const attachSubmenu = Submenu.new({
                                                        text: 'Open shell',
                                                        items: await Promise.all(attachItems)
                                                    });

                                                    itemPromises.push(logsSubmenu, attachSubmenu);
                                                }

                                                const items = await Promise.all(itemPromises);

                                                return Menu.new({ items });
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
