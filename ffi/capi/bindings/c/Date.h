#ifndef Date_H
#define Date_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "Calendar.d.h"
#include "CalendarDateAddError.d.h"
#include "CalendarDateFromFieldsError.d.h"
#include "CalendarError.d.h"
#include "DateAddOptions.d.h"
#include "DateDifferenceOptions.d.h"
#include "DateDuration.d.h"
#include "DateFields.d.h"
#include "DateFromFieldsOptions.d.h"
#include "IsoDate.d.h"
#include "Rfc9557ParseError.d.h"
#include "Weekday.d.h"

#include "Date.d.h"






typedef struct icu4x_Date_from_iso_in_calendar_mv1_result {union {Date ok; CalendarError err;}; bool is_ok;} icu4x_Date_from_iso_in_calendar_mv1_result;
icu4x_Date_from_iso_in_calendar_mv1_result icu4x_Date_from_iso_in_calendar_mv2(int32_t iso_year, uint8_t iso_month, uint8_t iso_day, const Calendar* calendar);

uint8_t icu4x_Date_day_of_month_mv2(Date self);

uint8_t icu4x_Date_month_number_mv2(Date self);

int32_t icu4x_Date_era_year_or_related_iso_mv2(Date self);


#endif // Date_H
