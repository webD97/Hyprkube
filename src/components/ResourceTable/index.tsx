import React from "react"

import { GenericResource } from "../../model/k8s";
import dayjs from "dayjs";

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

    return (
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
                    resources.sort(byCreationTimestamp).reverse().map(resource => (
                        <tr key={resource.metadata?.uid} onClick={() => onResourceClicked(resource)}>
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
    );
}

export default ResourceTable;
