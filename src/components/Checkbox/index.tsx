import { useCallback } from 'react';
import styles from './styles.module.css';

type CheckboxProps = React.DetailedHTMLProps<React.InputHTMLAttributes<HTMLInputElement>, HTMLInputElement>;

const Checkbox: React.FC<CheckboxProps> = (props) => {
    const stopPropagation: React.MouseEventHandler = useCallback((e) => e.stopPropagation(), []);

    return (
        <label className={styles.container} onClick={stopPropagation}>&nbsp;
            <input type="checkbox" {...props} />
            <span className={styles.checkmark}></span>
        </label>
    );
};

export default Checkbox;
