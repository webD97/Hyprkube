import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';

import { useEffect, useState } from 'react';
import { Gvk, NamespaceAndName } from './model/k8s';

import classes from './App.module.css';
import EmojiHint from './components/EmojiHint';
import GvkList from './components/GvkList';
import LogPanel from './components/LogPanel';
import ResourceView from './components/ResourceView';
import TabView, { Tab } from './components/TabView';
import { useTabs } from './components/TabView/hooks';
import StatusPanel from './containers/StatusPanel';
import { useClusterDiscovery } from './hooks/useClusterDiscovery';
import useResourceWatch from './hooks/useResourceWatch';
import NavHeader from './components/NavHeader';
import { KubeContextSource, useContextDiscovery } from './hooks/useContextDiscovery';
import { ClusterSelector } from './components/ClusetrSelector';

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

function App() {
  const contextSources = useContextDiscovery();
  const [currentGvk, setCurrentGvk] = useState<Gvk>();
  const [pinnedGvks, setPinnedGvks] = useState<Gvk[]>(defaultPinnedGvks);
  const [selectedResource, setSelectedResource] = useState<NamespaceAndName>({ namespace: '', name: '' });
  const [selectedView, setSelectedView] = useState("");
  const [selectedContext, setSelectedContext] = useState<KubeContextSource>();
  const { discovery, clientId } = useClusterDiscovery(selectedContext);
  const [columnTitles, resources] = useResourceWatch(clientId, currentGvk, selectedView);

  const [tabs, activeTab, pushTab, removeTab, setActiveTab] = useTabs();

  // If we have only 1 context, auto-select it
  useEffect(() => {
    if (contextSources.length !== 1) return;
    if (selectedContext !== undefined) return;

    setSelectedContext(contextSources[0]);
  }, [contextSources, selectedContext]);

  useEffect(() => {
    if (!currentGvk) return;

    const availableViews = discovery?.gvks[currentGvk.group].kinds.find(k => k.kind === currentGvk.kind)?.views || [];

    if (availableViews?.length < 1) return;

    setSelectedView(availableViews[0]);
  }, [selectedResource, currentGvk, discovery?.gvks]);

  dayjs.extend(relativeTime);

  return (
    <div className={classes.container}>
      <nav>
        <NavHeader />
        <hr />
        {
          selectedContext === undefined
            ? null
            : (
              <ClusterSelector
                selectedCluster={selectedContext}
                onSelect={(contextSource) => setSelectedContext(contextSource)}
                contextSources={contextSources}
              />
            )
        }
        <hr />
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
      <main className={classes.mainArea}>
        {
          selectedContext !== undefined
            ? null
            : (
              <ClusterSelector
                selectedCluster={selectedContext}
                onSelect={(contextSource) => setSelectedContext(contextSource)}
                contextSources={contextSources}
              />
            )
        }
        {
          currentGvk === undefined
            ? <EmojiHint emoji="🔍">Select a resource to get started.</EmojiHint>
            : (
              <>
                <div className={classes.topBar}>
                  <h2>{currentGvk?.kind}</h2>
                  <select value={selectedView} onChange={(e) => setSelectedView(e.target.value)}>
                    {
                      discovery?.gvks[currentGvk.group].kinds.find(v => v.kind === currentGvk.kind)?.views.map(view => (
                        <option key={view}>{view}</option>
                      ))
                    }
                  </select>
                </div>
                <ResourceView
                  columnTitles={columnTitles || []}
                  resourceData={resources}
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
      </main>
      <footer className={classes.appStatusBar}>
        <StatusPanel />
      </footer>
    </div>
  )
}

export default App
