import './App.css';

import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';

import useKubernetesNamespaceList from './hooks/useKubernetesNamespaceList';
import useKubernetesNodeList from './hooks/useKubernetesNodeList';
import useKubernetesResourceWatch from './hooks/useKubernetesResourceWatch';
import { KubernetesApiObject } from './model/k8s';
import { Pod } from 'kubernetes-types/core/v1';

function byCreationTimestamp(a: KubernetesApiObject, b: KubernetesApiObject) {
  const creationTimestampA = dayjs(a.metadata?.creationTimestamp);
  const creationTimestampB = dayjs(b.metadata?.creationTimestamp);

  return creationTimestampA.diff(creationTimestampB);
}

function App() {
  const nodes = useKubernetesNodeList();
  const namespaces = useKubernetesNamespaceList();
  const pods = useKubernetesResourceWatch<Pod>('kube_watch_pods');

  dayjs.extend(relativeTime);

  return (
    <>
      <header>
        <h1>Hyprkube</h1>
        <span>Namespace:&nbsp;</span>
        <select>
          {
            namespaces.map(namespace => (
              <option key={namespace.metadata?.uid}>
                {namespace.metadata?.name}
              </option>
            ))
          }
        </select>
      </header>
      <main>
        <h2>Nodes ({nodes.length})</h2>
        <table>
          <thead>
            <tr>
              <td>Name</td>
              <td>OS image</td>
              <td>Internal IP</td>
              <td>Architecture</td>
              <td>Kubelet version</td>
              <td>Age</td>
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
              <td>Name</td>
              <td>Namespace</td>
              <td>Host Integration</td>
              <td>Security Context</td>
              <td>Containers</td>
              <td>Restarts</td>
              <td>Node</td>
              <td>Age</td>
              <td>Status</td>
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
        </table>
      </main>
    </>
  )
}

export default App
