/**
 * Capitalize the first letter of a string. Only recommened to be used with short strings. Not perfect.
 * @see https://stackoverflow.com/a/53930826
 */
export function capitalizeFirstLetter([first = '', ...rest]: string) {
    return [first.toUpperCase(), ...rest].join('');
}
