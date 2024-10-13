import { GenericResource } from "../../model/k8s";
import dayjs from "dayjs";
import { JSONPath } from "jsonpath-plus";
import { coreNodes, corePods, genericNamespacedResource, genericNonNamespacedResource } from "./columns";

export interface ResourceTableProps<R extends GenericResource> {
    resources: R[],
    onResourceClicked?: (resource: R) => void,
}

function byCreationTimestamp(a: GenericResource, b: GenericResource) {
    const creationTimestampA = dayjs(a.metadata?.creationTimestamp);
    const creationTimestampB = dayjs(b.metadata?.creationTimestamp);

    return creationTimestampA.diff(creationTimestampB);
}

const ResourceTable = <R extends GenericResource>(props: ResourceTableProps<R>) => {
    const {
        resources,
        onResourceClicked = () => undefined
    } = props;

    const columns = (() => {
        if (resources.length === 0) return [];

        const { apiVersion, kind } = resources[0];

        if (apiVersion === 'v1' && kind === 'Pod') {
            return corePods;
        }
        if (apiVersion === 'v1' && kind === 'Node') {
            return coreNodes;
        }
        else if (resources[0]?.metadata?.namespace) {
            return genericNamespacedResource;
        }
        return genericNonNamespacedResource;
    })();

    return (
        <table>
            <thead>
                <tr>
                    {
                        columns.map(definition => definition[0]).map(name => (
                            <th key={name}>{name}</th>
                        ))
                    }
                </tr>
            </thead>
            <tbody>
                {
                    resources.sort(byCreationTimestamp).reverse().map(resource => (
                        <tr key={resource.metadata?.uid} onClick={() => onResourceClicked(resource)}>
                            {
                                columns.map(definition => definition[1]).map(({ jsonPath, transform = (value) => value }) => (
                                    <td key={jsonPath}>
                                        {
                                            (() => {
                                                const value = transform(
                                                    JSONPath({
                                                        path: jsonPath,
                                                        json: resource
                                                    }));
                                                
                                                if (typeof (value) === 'string') {
                                                    return (<span>{value}</span>);
                                                }

                                                return value;
                                            })()
                                        }
                                    </td>
                                ))
                            }
                        </tr>
                    ))
                }
            </tbody>
        </table>
    );
}

export default ResourceTable;
