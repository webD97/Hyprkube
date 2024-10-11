import './App.css';

import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';

import useKubernetesResourceWatch from './hooks/useKubernetesResourceWatch';
import { KubernetesApiObject } from './model/k8s';
import { Node, Pod } from 'kubernetes-types/core/v1';
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

function byCreationTimestamp(a: KubernetesApiObject, b: KubernetesApiObject) {
  const creationTimestampA = dayjs(a.metadata?.creationTimestamp);
  const creationTimestampB = dayjs(b.metadata?.creationTimestamp);

  return creationTimestampA.diff(creationTimestampB);
}

function App() {
  const nodes = useKubernetesResourceWatch<Node>('', 'v1', 'Node');
  const pods = useKubernetesResourceWatch<Pod>('', 'v1', 'Pod');

  const [gvks, setGvks] = useState<{ [key: string]: [string, string] }>({});

  useEffect(() => {
    invoke("kube_discover").then(result => {
      setGvks(result as typeof gvks)
    });
  }, []);

  dayjs.extend(relativeTime);

  return (
    <div className="container">
      <nav>
        <h1>🧊&nbsp;Hyprkube</h1>
        <h2>Pinned resources</h2>
        <ul className="pinned">
          <li>Pods</li>
          <li>Deployments</li>
          <li>Clusters (fleet.cattle.io)</li>
          <li>BundleDeployments (fleet.cattle.io)</li>
          <li>GitJob (gitjob.cattle.io)</li>
        </ul>
        <h2>All resources</h2>
        {
          Object.entries(gvks).map(([g, vk]) => (
            <details key={g}>
              <summary>{g ? g : 'core'}</summary>
              <ul>
                {
                  vk.map(([k, v]) => (
                    <li key={`${k}/${v}`}>{k}</li>
                  ))
                }
              </ul>
            </details>
          ))
        }
      </nav>
      <main>
        <aside>
          <p className="icon">🔍</p>
          <p>Select a resource</p>
        </aside>
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
