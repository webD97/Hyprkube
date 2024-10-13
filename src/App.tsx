import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';

import useKubernetesResourceWatch from './hooks/useKubernetesResourceWatch';
import { Gvk, NamespaceAndName, KubernetesClient } from './model/k8s';
import { useEffect, useState } from 'react';

import classes from './App.module.css';
import GvkList from './components/GvkList';
import LogPanel from './components/LogPanel';
import { getDefaultKubernetesClient } from './api/KubernetesClient';
import { useGvks } from './hooks/useGvks';
import ResourceTable from './components/ResourceTable';

const defaultPinnedGvks: Gvk[] = [
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
  const currentResourceList = useKubernetesResourceWatch(kubernetesClient, currentGvk);

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

        <h2>All resources</h2>
        {
          Object.entries(gvks)
            .sort(([groupA], [groupB]) => groupA.localeCompare(groupB))
            .map(([group, vks]) => {
              const gvks: Gvk[] = vks.map(([kind, version]) => ({ group, version, kind }));

              return (
                <details key={group}>
                  <summary>{group ? group : 'core'}</summary>
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
      {
        currentGvk?.kind === 'Pod' && selectedResource?.namespace!! && selectedResource?.name!!
          ? (
            <section className={classes.bottomPanel}>
              <LogPanel kubernetesClient={kubernetesClient} namespace={selectedResource.namespace} name={selectedResource.name} />
            </section>
          )
          : null
      }
      <main className={classes.mainArea}>
        {
          currentGvk === undefined
            ? (
              <aside className={classes.miniHint}>
                <p>🔍</p>
                <p>Select a resource</p>
              </aside>
            )
            : (
              <>
                <h2>{currentGvk?.kind} ({currentResourceList.length})</h2>
                <ResourceTable
                  resources={currentResourceList}
                  onResourceClicked={(resource) => setSelectedResource({ namespace: resource.metadata?.namespace, name: resource.metadata?.name })}
                />
              </>
            )
        }

        {/* <h2>Nodes ({nodes.length})</h2>
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>OS image</th>
              <th>Internal IP</th>
              <th>Architecture</th>
              <th>Kubelet version</th>
              <th>Age</th>
            </tr>
          </thead>
          <tbody>
            {
              nodes.map(node => (
                <tr key={node.metadata?.uid}>
                  <td>{node.metadata?.name}</td>
                  <td>{node.status?.nodeInfo?.osImage}</td>
                  <td>
                    {
                      node.status?.addresses?.find((address: any) => address.type === 'InternalIP')?.address
                    }
                  </td>
                  <td>{node.status?.nodeInfo?.architecture}</td>
                  <td>{node.status?.nodeInfo?.kubeletVersion}</td>
                  <td>
                    {dayjs().to(dayjs(node.metadata?.creationTimestamp), true)}
                  </td>
                </tr>
              ))
            }
          </tbody>
        </table>*/}

      </main>
    </div>
  )
}

export default App
