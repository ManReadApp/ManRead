export function calculateSliderReduce(screen: number, max_width: number) {
    return (max_width / screen) * 50
}

export function calculateSliderValueMax(screen: number, reduce_width: number) {
    return 50 - ((reduce_width / screen) * 50)
}

export function calculateSliderPercentage(percentage: number) {
    return 50 - (percentage / 2);
}

export function calculatePxValueReduce(screen: number, value: number) {
    return screen * (value * 2 * 0.01)
}

export function calculatePxValueMax(screen: number, value: number) {
    return screen * ((50 - value) * 2 * 0.01)
}

export function calculatePercentage(value: number) {
    return (50 - value) * 2;
}