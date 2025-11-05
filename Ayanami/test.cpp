#include <iostream>
using namespace std;

void swapintint(int& i,int& j) {
int temp;
temp = i;
i = j;
j = temp;
}

void sortintintARR_int(int start,int end,int* arr) {
if_start1:
bool t4;
t4 = start > end;
bool t5;
t5 = start == end;
bool t6;
t6 = t4 || t5;
if(!(t6)) goto if_end2;
return ;
if_end2:
int i;
i = start;
int j;
j = end;
int t9;
t9 = i + j;
int t10;
t10 = t9 / 2.000000;
int pivot;
pivot = arr[t10];
while_start3:
int t12;
t12 = i + 1.000000;
bool t13;
t13 = t12 < j;
if(!(t13)) goto while_end4;
while_start5:
bool t14;
t14 = arr[i] < pivot;
if(!(t14)) goto while_end6;
int t15;
t15 = i + 1.000000;
i = t15;
goto while_start5;
while_end6:
while_start7:
bool t17;
t17 = pivot < arr[j];
if(!(t17)) goto while_end8;
int t18;
t18 = j - 1.000000;
j = t18;
goto while_start7;
while_end8:
if_start9:
int t20;
t20 = i + 1.000000;
bool t21;
t21 = t20 < j;
if(!(t21)) goto if_end10;
swapintint(arr[i],arr[j]);
int t23;
t23 = i + 1.000000;
i = t23;
int t25;
t25 = j - 1.000000;
j = t25;
if_end10:
goto while_start3;
while_end4:
if_start11:
bool t27;
t27 = start < j;
if(!(t27)) goto if_end12;
sortintintARR_int(start,j,arr);
if_end12:
if_start13:
bool t29;
t29 = i < end;
if(!(t29)) goto if_end14;
sortintintARR_int(i,end,arr);
if_end14:
}

void printARR_int(int* arr) {
int t31;
t31 = 0.000000;
for_start15:
int t32;
t32 = 9.000000 - t31;
if(!(t32)) goto for_end16;
int i;
i = t31;
std::cout<<arr[i];
std::cout<<' ';
int t33;
t33 = t31 + 1.000000;
t31 = t33;
goto for_start15;
for_end16:
}

int main() {
int* t34 = new int[9];
t34[0] = 1.000000;
t34[1] = 6.000000;
t34[2] = 3.000000;
t34[3] = 98.000000;
t34[4] = 2.000000;
t34[5] = 4.000000;
t34[6] = 6.000000;
t34[7] = 1.000000;
t34[8] = 89.000000;
int* arr;
arr = t34;
sortintintARR_int(0.000000,8.000000,arr);
printARR_int(arr);
}

