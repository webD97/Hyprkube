import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';

import useKubernetesResourceWatch from './hooks/useKubernetesResourceWatch';
import { Gvk, GenericResource } from './model/k8s';
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

import classes from './App.module.css';
import GvkList from './components/GvkList';

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

function byCreationTimestamp(a: GenericResource, b: GenericResource) {
  const creationTimestampA = dayjs(a.metadata?.creationTimestamp);
  const creationTimestampB = dayjs(b.metadata?.creationTimestamp);

  return creationTimestampA.diff(creationTimestampB);
}

function App() {
  const [gvks, setGvks] = useState<{ [key: string]: [string, string] }>({});
  const [currentGvk, setCurrentGvk] = useState<Gvk>();
  const [pinnedGvks, setPinnedGvks] = useState<Gvk[]>(defaultPinnedGvks);
  const currentResourceList = useKubernetesResourceWatch(currentGvk);

  useEffect(() => {
    invoke("kube_discover").then(result => {
      setGvks(result as typeof gvks)
    });
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
      <main>
        {
          currentGvk === undefined
            ? (
              <aside>
                <p className="icon">🔍</p>
                <p>Select a resource</p>
              </aside>
            )
            : (
              <>
                <h2>{currentGvk?.kind} ({currentResourceList.length})</h2>
                <table>
                  <thead>
                    <tr>
                      <th>Name</th>
                      <th>Namespace</th>
                      <th>Age</th>
                    </tr>
                  </thead>
                  <tbody>
                    {
                      currentResourceList.sort(byCreationTimestamp).reverse().map(resource => (
                        <tr key={resource.metadata?.uid}>
                          <td>{resource.metadata?.name}</td>
                          <td>{resource.metadata?.namespace}</td>
                          <td>
                            {dayjs().to(dayjs(resource.metadata?.creationTimestamp), true)}
                          </td>
                        </tr>
                      ))
                    }
                  </tbody>
                </table>
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
        </table>
        <h2>Pods ({pods.length})</h2>
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Namespace</th>
              <th>Host Integration</th>
              <th>Security Context</th>
              <th>Containers</th>
              <th>Restarts</th>
              <th>Node</th>
              <th>Age</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            {
              pods.sort(byCreationTimestamp).reverse().map(pod => (
                <tr key={pod.metadata?.uid}>
                  <td>{pod.metadata?.name}</td>
                  <td>{pod.metadata?.namespace}</td>
                  <td>
                    {
                      [
                        (pod.spec?.hostNetwork && 'N'),
                        (pod.spec?.hostPID && 'P'),
                        (pod.spec?.hostIPC && 'I'),
                      ].filter(flag => flag !== undefined).join(', ')
                    }
                  </td>
                  <td>
                    {
                      [
                        (pod.spec?.containers.find(c => c.securityContext?.privileged) && 'P'),
                        (pod.spec?.containers.find(c => !c.securityContext?.readOnlyRootFilesystem) && 'W'),
                      ].filter(flag => flag !== undefined).join(', ')
                    }
                  </td>
                  <td>
                    {pod.status?.containerStatuses?.map(status => status.ready).map(ready => ready ? '▪' : '▫')}
                  </td>
                  <td>{pod.status?.containerStatuses?.map(status => status.restartCount).reduce((previous, next) => previous + next)}</td>
                  <td>{pod.spec?.nodeName}</td>
                  <td>
                    {dayjs().to(dayjs(pod.metadata?.creationTimestamp), true)}
                  </td>
                  <td>{pod.status?.phase}</td>
                </tr>
              ))
            }
          </tbody>
        </table> */}

      </main>
    </div>
  )
}

export default App
