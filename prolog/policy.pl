% policy.pl 
:- module(policy,[
    % list all preds    
    monthlyConsumptions/1,
    sumOfMonthlyConsumptions/2,
    rollingConsumption/2,
    consumptionClass/2,
    savingsClass/3,
    priceBase/1,
    applySupport/4,
    applySavingsSupport/4,
    socialCreds/1,
    applySocialSupports/3,
    endPrice/1,
    inputPriceOk/0
    ]).

% Flow: 
% 1. sum individual past consumptions
% 2. calculate rolling consumption
% 3. classify using rolling consumpiton
% 4. classify using currently saved amount compared to rollingconsumption
% 5. apply savings based support to base payement
% 6. apply social credential based support 

% === 1. Aggregate past consumptions ===

% monthlyConsumptions(MonthlyConsumptions) :- findall((Amount), monthly_consumption(_,Amount), MonthlyConsumptions).
% monthlyConsumptions(MonthlyConsumptions) :- MonthlyConsumptions = [(1,2001),(2,2001),(3,2001),(4,2001),(5,2001),(6,2001),(7,2000),(8,2000),(9,2000),(10,2000),(11,2000),(12,2000)].
monthlyConsumptions(MonthlyConsumptions) :- MonthlyConsumptions = [2001,2001,2001,2001,2001,2001,2000,2000,2000,2000,2000,2000].

sumOfMonthlyConsumptions([Amount|Tail],Sum) :- sumOfMonthlyConsumptions(Tail,SumOfTail), Sum is SumOfTail + Amount.
sumOfMonthlyConsumptions([],0).

% === 2. Calculate rolling consumption ===
rollingConsumption(Sum,Result):- 
    Result is Sum div 12.

% === 3. classify using rolling consumption ===   
consumptionClass(RollingConsumptionVar,Class):-
    rolling_treshold('high',Treshold),
    RollingConsumptionVar > Treshold,
    Class = 'high'.
consumptionClass(RollingConsumptionVar,Class):-
    rolling_treshold('mid',Treshold),
    RollingConsumptionVar > Treshold,
    Class = 'mid'.
consumptionClass(_RollingConsumptionVar,Class):-
    Class = 'low'.

% === 4. classify using savings ===
savingsClass(RollingConsumptionVar,Consumption,Class):-
    savings_treshold('high',Treshold),
    CurrentSaving is RollingConsumptionVar - Consumption,
    CurrentSaving > Treshold,
    Class = 'high';
    savings_treshold('mid',Treshold),
    CurrentSaving is RollingConsumptionVar - Consumption,
    CurrentSaving > Treshold,
    Class = 'mid';
    savings_treshold('low',Treshold),
    CurrentSaving is RollingConsumptionVar - Consumption,
    CurrentSaving > Treshold,
    Class = 'low';
    Class = 'none'.

% === 5. Apply savings based support ===
priceBase(PriceBase):-
    currentConsumption(Amount),
    currentPrice(Price,'HUF'),
    PriceBase is Price * Amount.

applySupport(ApplySupportInput, nominal, ApplySupportValue, ApplySupportOutput):-
    ApplySupportOutput is ApplySupportInput - ApplySupportValue.
applySupport(ApplySupportInput, percent, ApplySupportValue, ApplySupportOutput):-
    AmountToBeSubtracted is ApplySupportInput * ApplySupportValue div 100,
    ApplySupportOutput is ApplySupportInput - AmountToBeSubtracted.

applySavingsSupport(ApplySavingsSupportInput, ApplySavingsSupportSavingsClass, ApplySavingsSupportRollingClass, ApplySavingsSupportOutput):-
    support_matrix(ApplySavingsSupportRollingClass, ApplySavingsSupportSavingsClass, ApplySavingsSupportType, ApplySavingsSupportValue),
    applySupport(ApplySavingsSupportInput, ApplySavingsSupportType, ApplySavingsSupportValue, ApplySavingsSupportOutput).

% === 6. Apply social standing based support ===
socialCreds(Creds):- Creds = [('ChangedWorkcapacityCredential',nominal,10000)].
% socialCreds(Creds) :- findall((CredType,SupportType,SupportValue), social_suport(CredType,SupportType,SupportValue), Creds).

applySocialSupports(Input,[(_,SupportType,SupportValue)|CredsTail],Result):-
    applySupport(Input, SupportType, SupportValue,Output),
    applySocialSupports(Output,CredsTail,Result).
applySocialSupports(Input,[], Input).

endPrice(Price):-
    monthlyConsumptions(MonthlyConsumptions),
    sumOfMonthlyConsumptions(MonthlyConsumptions,Sum),
    rollingConsumption(Sum,RollingConsumptionVar),
    currentConsumption(Consumption),
    consumptionClass(RollingConsumptionVar,ConsumptionClassVar),
    savingsClass(RollingConsumptionVar,Consumption,SavingsClass),
    priceBase(PriceBase),
    applySavingsSupport(PriceBase, SavingsClass, ConsumptionClassVar, PriceAfterSavings),
    socialCreds(Creds),
    applySocialSupports(PriceAfterSavings,Creds,Price).

inputPriceOk:-
    endPrice(Price),
    inputPayment(Price).