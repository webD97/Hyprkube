import { useCallback } from 'react';
import styles from './styles.module.css';

type CheckboxProps = { label?: string } & React.DetailedHTMLProps<React.InputHTMLAttributes<HTMLInputElement>, HTMLInputElement>;

const Checkbox: React.FC<CheckboxProps> = (props) => {
    const stopPropagation: React.MouseEventHandler = useCallback((e) => e.stopPropagation(), []);

    return (
        <span className={`${styles.container} ${props.label ? styles.withLabel : ''}`}>
            <label onClick={stopPropagation}>{props.label ? props.label : null}&nbsp;
                <input type="checkbox" {...props} />
                <span className={styles.checkmark}></span>
            </label>
        </span>
    );
};

export default Checkbox;
