// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

#include "CalendarKind.h"
#include "Calendar.h"
#include "Date.h"
#include "Logger.h"
#include <string.h>
#include <stdio.h>

int main()
{
    icu4x_Logger_init_simple_logger_mv1();

    Calendar *calendar = icu4x_Calendar_create_mv1(CalendarKind_Coptic);

    Date date = icu4x_Date_from_iso_in_calendar_mv2(2026, 2, 27, calendar).ok;

    printf(
        "Coptic day: %d-%d-%d\n", 
        icu4x_Date_day_of_month_mv2(date), 
        icu4x_Date_month_number_mv2(date), 
        icu4x_Date_era_year_or_related_iso_mv2(date)
    );

    return 0;
}
