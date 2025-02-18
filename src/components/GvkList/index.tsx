import React from 'react';
import { Gvk } from '../../model/k8s';

import classes from './component.module.css';
import { Menu } from '@tauri-apps/api/menu';
import { PhysicalPosition } from '@tauri-apps/api/dpi';

export interface GvkListProps {
    className?: string,
    gvks: Gvk[],
    withGroupName?: boolean,
    onResourceClicked?: (gvk: Gvk) => void,
    onGvkContextMenu: (gvk: Gvk) => Promise<Menu>,
}

const GvkList: React.FC<GvkListProps> = (props) => {
    const {
        className,
        gvks,
        withGroupName = false,
        onGvkContextMenu,
        onResourceClicked = () => undefined,
    } = props;

    return (
        <div className={[classes.container, className].filter(c => !!c).join(' ')}>
            <ul>
                {
                    gvks.map((gvk) => {
                        const { group, version, kind } = gvk;

                        return (
                            <li key={`${kind}.${group}/${version}`}>
                                <span
                                    onClick={() => onResourceClicked(gvk)}
                                    onContextMenu={e => {
                                        e.preventDefault();
                                        onGvkContextMenu(gvk)
                                            .then(menu => menu.popup(new PhysicalPosition(e.screenX, e.screenY)))
                                            .catch(e => alert(JSON.stringify(e)));
                                    }
                                    }>
                                    {
                                        group !== '' && withGroupName
                                            ? `${kind} (${group})`
                                            : `${kind}`
                                    }
                                </span>
                            </li>
                        );
                    })
                }
            </ul>
        </div>
    );
};

export default GvkList;
