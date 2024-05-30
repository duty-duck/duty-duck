const steps = [
    [30, "30 Seconds"],
    [60, "1 minute"],
    ...Array.from({length: 11}, (x, i) => [(i + 1) * 5 * 60, `${(i + 1) * 5} minutes`]),
    [3600, "1 hour"],
    ...Array.from({length: 22}, (x, i) => [(i + 2) * 3600, `${i + 2} hours`]),
    [3600 * 25, "1 day"]
];

export default (initialIntervalSeconds) => {
    let currentStepIndex = 1;
    if (initialIntervalSeconds) {
        currentStepIndex = steps.findIndex(step => step[0] == initialIntervalSeconds)
    }
    return {
        currentStepIndex,
        min: 0,
        max: steps.length - 1,
        get currentStep() {
            return steps[this.currentStepIndex]
        }
    }
}
