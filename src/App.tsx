import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';

import { Gvk, KubernetesClient, NamespaceAndName } from './model/k8s';
import { useEffect, useState } from 'react';

import classes from './App.module.css';
import GvkList from './components/GvkList';
import { getDefaultKubernetesClient } from './api/KubernetesClient';
import { useGvks } from './hooks/useGvks';
import EmojiHint from './components/EmojiHint';
import useResourceWatch from './hooks/useResourceWatch';
import ResourceView from './components/ResourceView';
import LogPanel from './components/LogPanel';
import StatusPanel from './containers/StatusPanel';

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
  const [kubernetesClient, setKubernetesClient] = useState<KubernetesClient | undefined>(undefined);
  const gvks = useGvks(kubernetesClient);
  const [currentGvk, setCurrentGvk] = useState<Gvk>();
  const [pinnedGvks, setPinnedGvks] = useState<Gvk[]>(defaultPinnedGvks);
  const [selectedResource, setSelectedResource] = useState<NamespaceAndName>({ namespace: '', name: '' });
  const [selectedView, setSelectedView] = useState("");
  const [columnTitles, resources] = useResourceWatch(kubernetesClient, currentGvk, selectedView);


  useEffect(() => {
    if (!currentGvk) return;

    const availableViews = gvks?.gvks[currentGvk.group].kinds.find(k => k.kind === currentGvk.kind)?.views || [];

    if (availableViews?.length < 1) return;

    setSelectedView(availableViews[0]);
  }, [selectedResource, currentGvk, gvks?.gvks]);

  useEffect(() => {
    getDefaultKubernetesClient()
      .then(client => setKubernetesClient(client))
      .catch(e => alert(e));
  }, []);

  dayjs.extend(relativeTime);

  return (
    <div className={classes.container}>
      <nav>
        <h1>🧊&nbsp;Hyprkube</h1>
        {
          pinnedGvks.length == 0
            ? null
            : (
              <>
                <h2>Pinned resources</h2>
                <GvkList withGroupName
                  gvks={pinnedGvks}
                  onResourceClicked={(gvk) => setCurrentGvk(gvk)}
                  onPinButtonClicked={({ group, kind }) => setPinnedGvks(currentlyPinned => currentlyPinned.filter(clickedGvk => clickedGvk.group !== group || clickedGvk.kind !== kind))}
                />
              </>
            )
        }

        <h2>Builtin resources</h2>
        {
          Object.values(gvks?.gvks || [])
            .filter((group) => !group.isCrd)
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

        <h2>Custom resources</h2>
        {
          Object.values(gvks?.gvks || [])
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
        {
          currentGvk?.kind === 'Pod' && selectedResource?.namespace && selectedResource?.name
            ? (
              <LogPanel kubernetesClient={kubernetesClient} namespace={selectedResource.namespace} name={selectedResource.name} />
            )
            : null
        }
      </section>
      <main className={classes.mainArea}>
        {
          currentGvk === undefined
            ? <EmojiHint emoji="🔍">Select a resource to get started.</EmojiHint>
            : (
              <>
                <div className={classes.topBar}>
                  <h2>{currentGvk?.kind}</h2>
                  <select value={selectedView} onChange={(e) => setSelectedView(e.target.value)}>
                    {
                      gvks?.gvks[currentGvk.group].kinds.find(v => v.kind === currentGvk.kind)?.views.map(view => (
                        <option key={view}>{view}</option>
                      ))
                    }
                  </select>
                </div>
                <ResourceView
                  columnTitles={columnTitles || []}
                  resourceData={resources}
                  onResourceClicked={(uid) => {
                    const resource = resources[uid];
                    setSelectedResource({
                      namespace: resource.namespace,
                      name: resource.name
                    });
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
